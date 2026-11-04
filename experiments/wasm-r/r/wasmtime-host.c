#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#elif !defined(_POSIX_C_SOURCE)
#define _POSIX_C_SOURCE 200809L
#endif

#include <R.h>
#include <Rinternals.h>
#include <wasmtime.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#if defined(_WIN32)
#include <windows.h>
#include <psapi.h>
#else
#include <sys/resource.h>
#include <time.h>
#endif

/* Keep one Wasmtime engine/store/instance alive for repeated .Call invocations. */
static wasm_engine_t *g_engine = NULL;
static wasmtime_store_t *g_store = NULL;
static wasmtime_context_t *g_context = NULL;
static wasmtime_instance_t g_instance;
static wasmtime_extern_t g_memory;
static wasmtime_func_t g_alloc;
static wasmtime_func_t g_free;
static wasmtime_func_t g_validate;
static char *g_module_path = NULL;

static void fail_wasmtime(const char *where, wasmtime_error_t *wt_error, wasm_trap_t *trap) {
  wasm_byte_vec_t message;
  if (wt_error != NULL) {
    wasmtime_error_message(wt_error, &message);
    wasmtime_error_delete(wt_error);
  } else {
    wasm_trap_message(trap, &message);
    wasm_trap_delete(trap);
  }
  Rf_error("%s: %.*s", where, (int)message.size, message.data);
}

static void call_export(wasmtime_func_t *function,
                        wasmtime_val_t *args, size_t nargs,
                        wasmtime_val_t *results, size_t nresults,
                        const char *name) {
  wasm_trap_t *trap = NULL;
  wasmtime_error_t *wt_error = wasmtime_func_call(
      g_context, function, args, nargs, results, nresults, &trap);
  if (wt_error != NULL || trap != NULL) fail_wasmtime(name, wt_error, trap);
}

static void shutdown_host(void) {
  if (g_store != NULL) wasmtime_store_delete(g_store);
  if (g_engine != NULL) wasm_engine_delete(g_engine);
  free(g_module_path);
  g_engine = NULL;
  g_store = NULL;
  g_context = NULL;
  g_module_path = NULL;
}

static void ensure_module(const char *module_path) {
  if (g_context != NULL && g_module_path != NULL &&
      strcmp(g_module_path, module_path) == 0) return;
  shutdown_host();

  FILE *file = fopen(module_path, "rb");
  if (file == NULL) Rf_error("cannot open WASM module: %s", module_path);
  if (fseek(file, 0, SEEK_END) != 0) Rf_error("cannot seek WASM module");
  long file_len = ftell(file);
  if (file_len <= 0 || fseek(file, 0, SEEK_SET) != 0) Rf_error("invalid WASM module size");
  uint8_t *bytes = (uint8_t *)malloc((size_t)file_len);
  if (bytes == NULL) Rf_error("out of memory reading WASM module");
  if (fread(bytes, 1, (size_t)file_len, file) != (size_t)file_len)
    Rf_error("cannot read WASM module");
  fclose(file);

  g_engine = wasm_engine_new();
  g_store = wasmtime_store_new(g_engine, NULL, NULL);
  g_context = wasmtime_store_context(g_store);
  wasmtime_module_t *module = NULL;
  wasmtime_error_t *wt_error = wasmtime_module_new(g_engine, bytes, (size_t)file_len, &module);
  free(bytes);
  if (wt_error != NULL) fail_wasmtime("compile WASM module", wt_error, NULL);

  wasm_trap_t *trap = NULL;
  wt_error = wasmtime_instance_new(g_context, module, NULL, 0, &g_instance, &trap);
  if (wt_error != NULL || trap != NULL) fail_wasmtime("instantiate WASM module", wt_error, trap);
  wasmtime_module_delete(module);

  wasmtime_extern_t item;
  if (!wasmtime_instance_export_get(g_context, &g_instance, "memory", 6, &g_memory) ||
      g_memory.kind != WASMTIME_EXTERN_MEMORY) Rf_error("WASM module must export memory");
  if (!wasmtime_instance_export_get(g_context, &g_instance, "corrplot_alloc", 14, &item) ||
      item.kind != WASMTIME_EXTERN_FUNC) Rf_error("WASM module must export corrplot_alloc");
  g_alloc = item.of.func;
  if (!wasmtime_instance_export_get(g_context, &g_instance, "corrplot_free", 13, &item) ||
      item.kind != WASMTIME_EXTERN_FUNC) Rf_error("WASM module must export corrplot_free");
  g_free = item.of.func;
  if (!wasmtime_instance_export_get(g_context, &g_instance, "corrplot_validate_json", 22, &item) ||
      item.kind != WASMTIME_EXTERN_FUNC) Rf_error("WASM module must export corrplot_validate_json");
  g_validate = item.of.func;

  g_module_path = (char *)malloc(strlen(module_path) + 1);
  if (g_module_path == NULL) Rf_error("out of memory caching module path");
  strcpy(g_module_path, module_path);
}

typedef struct {
  uint64_t alloc_ns;
  uint64_t input_copy_ns;
  uint64_t guest_execution_ns;
  uint64_t input_free_ns;
  uint64_t output_copy_ns;
  uint64_t output_free_ns;
} phase_timings_t;

static uint64_t monotonic_ns(void) {
#if defined(_WIN32)
  LARGE_INTEGER counter, frequency;
  QueryPerformanceCounter(&counter);
  QueryPerformanceFrequency(&frequency);
  return (uint64_t)((double)counter.QuadPart * 1000000000.0 / (double)frequency.QuadPart);
#else
  struct timespec value;
  if (clock_gettime(CLOCK_MONOTONIC, &value) != 0) return 0;
  return (uint64_t)value.tv_sec * UINT64_C(1000000000) + (uint64_t)value.tv_nsec;
#endif
}

static char *validate_json(const char *module_path, const char *json,
                           phase_timings_t *timings, size_t *output_len) {
  size_t json_len = strlen(json);
  ensure_module(module_path);

  wasmtime_val_t args[2], results[1];
  uint64_t started = monotonic_ns();
  args[0].kind = WASMTIME_I32;
  args[0].of.i32 = (int32_t)json_len;
  call_export(&g_alloc, args, 1, results, 1, "call corrplot_alloc");
  uint32_t input_ptr = (uint32_t)results[0].of.i32;
  if (timings != NULL) timings->alloc_ns = monotonic_ns() - started;

  uint8_t *memory = wasmtime_memory_data(g_context, &g_memory.of.memory);
  size_t memory_len = wasmtime_memory_data_size(g_context, &g_memory.of.memory);
  if (input_ptr == 0 || input_ptr > memory_len || json_len > memory_len - input_ptr)
    Rf_error("WASM input buffer is outside linear memory");

  started = monotonic_ns();
  memcpy(memory + input_ptr, json, json_len);
  if (timings != NULL) timings->input_copy_ns = monotonic_ns() - started;

  args[0].kind = WASMTIME_I32; args[0].of.i32 = (int32_t)input_ptr;
  args[1].kind = WASMTIME_I32; args[1].of.i32 = (int32_t)json_len;
  started = monotonic_ns();
  call_export(&g_validate, args, 2, results, 1, "call corrplot_validate_json");
  if (timings != NULL) timings->guest_execution_ns = monotonic_ns() - started;
  uint64_t packed = (uint64_t)results[0].of.i64;

  args[0].kind = WASMTIME_I32; args[0].of.i32 = (int32_t)input_ptr;
  args[1].kind = WASMTIME_I32; args[1].of.i32 = (int32_t)json_len;
  started = monotonic_ns();
  call_export(&g_free, args, 2, NULL, 0, "free WASM input");
  if (timings != NULL) timings->input_free_ns = monotonic_ns() - started;

  uint32_t output_ptr = (uint32_t)(packed & UINT64_C(0xffffffff));
  uint32_t output_size = (uint32_t)(packed >> 32);
  memory = wasmtime_memory_data(g_context, &g_memory.of.memory);
  memory_len = wasmtime_memory_data_size(g_context, &g_memory.of.memory);
  if (output_ptr == 0 || output_size == 0 || output_ptr > memory_len || output_size > memory_len - output_ptr)
    Rf_error("WASM output buffer is outside linear memory");

  started = monotonic_ns();
  char *output = (char *)R_alloc((size_t)output_size + 1, 1);
  memcpy(output, memory + output_ptr, output_size);
  output[output_size] = '\0';
  if (timings != NULL) timings->output_copy_ns = monotonic_ns() - started;
  if (output_len != NULL) *output_len = output_size;

  args[0].kind = WASMTIME_I32; args[0].of.i32 = (int32_t)output_ptr;
  args[1].kind = WASMTIME_I32; args[1].of.i32 = (int32_t)output_size;
  started = monotonic_ns();
  call_export(&g_free, args, 2, NULL, 0, "free WASM output");
  if (timings != NULL) timings->output_free_ns = monotonic_ns() - started;
  return output;
}

SEXP corrplot_wasm_validate(SEXP module_path_sexp, SEXP json_sexp) {
  phase_timings_t timings = {0};
  char *output = validate_json(CHAR(STRING_ELT(module_path_sexp, 0)),
                               CHAR(STRING_ELT(json_sexp, 0)), &timings, NULL);
  return Rf_mkString(output);
}

SEXP corrplot_wasm_validate_timed(SEXP module_path_sexp, SEXP json_sexp) {
  phase_timings_t timings = {0};
  char *output = validate_json(CHAR(STRING_ELT(module_path_sexp, 0)),
                               CHAR(STRING_ELT(json_sexp, 0)), &timings, NULL);
  const char *labels[] = {"response", "allocNs", "inputCopyNs", "guestExecutionNs",
                          "inputFreeNs", "outputCopyNs", "outputFreeNs"};
  double values[] = {(double)timings.alloc_ns, (double)timings.input_copy_ns,
                     (double)timings.guest_execution_ns, (double)timings.input_free_ns,
                     (double)timings.output_copy_ns, (double)timings.output_free_ns};
  SEXP result = PROTECT(Rf_allocVector(VECSXP, 7));
  SEXP names = PROTECT(Rf_allocVector(STRSXP, 7));
  SET_VECTOR_ELT(result, 0, Rf_mkString(output));
  for (int i = 0; i < 6; i++) {
    SET_STRING_ELT(names, i + 1, Rf_mkChar(labels[i + 1]));
    SET_VECTOR_ELT(result, i + 1, Rf_ScalarReal(values[i]));
  }
  SET_STRING_ELT(names, 0, Rf_mkChar(labels[0]));
  Rf_setAttrib(result, R_NamesSymbol, names);
  UNPROTECT(2);
  return result;
}

static double peak_rss_bytes(void) {
#if defined(_WIN32)
  PROCESS_MEMORY_COUNTERS counters;
  counters.cb = sizeof(counters);
  if (!GetProcessMemoryInfo(GetCurrentProcess(), &counters, sizeof(counters))) return NA_REAL;
  return (double)counters.PeakWorkingSetSize;
#else
  struct rusage usage;
  if (getrusage(RUSAGE_SELF, &usage) != 0) return NA_REAL;
#if defined(__APPLE__)
  /* Darwin's struct rusage does not expose ru_maxrss. Do not substitute a
   * current-RSS snapshot while labeling it a process-lifetime high-water mark. */
  return NA_REAL;
#else
  return (double)usage.ru_maxrss * 1024.0;
#endif
#endif
}

SEXP corrplot_wasm_peak_rss_bytes(void) {
  return Rf_ScalarReal(peak_rss_bytes());
}


SEXP corrplot_wasm_shutdown(void) {
  shutdown_host();
  return R_NilValue;
}


/* Explicit initialization lets the R runner separate module startup from its
 * first validation call. These helpers are intentionally read-only. */
SEXP corrplot_wasm_initialize(SEXP module_path_sexp) {
  const char *module_path = CHAR(STRING_ELT(module_path_sexp, 0));
  ensure_module(module_path);
  return R_NilValue;
}

SEXP corrplot_wasm_memory_bytes(void) {
  if (g_context == NULL) Rf_error("WASM module is not initialized");
  return Rf_ScalarReal((double)wasmtime_memory_data_size(g_context, &g_memory.of.memory));
}
