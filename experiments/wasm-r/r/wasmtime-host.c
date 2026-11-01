#include <R.h>
#include <Rinternals.h>
#include <wasmtime.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void fail_wasmtime(const char *where, wasmtime_error_t *error, wasm_trap_t *trap) {
  wasm_byte_vec_t message;
  if (error != NULL) {
    wasmtime_error_message(error, &message);
    wasmtime_error_delete(error);
  } else {
    wasm_trap_message(trap, &message);
    wasm_trap_delete(trap);
  }
  Rf_error("%s: %.*s", where, (int)message.size, message.data);
  wasm_byte_vec_delete(&message);
}

static void call_export(wasmtime_context_t *context, wasmtime_func_t *function,
                        wasmtime_val_t *args, size_t nargs,
                        wasmtime_val_t *results, size_t nresults,
                        const char *name) {
  wasm_trap_t *trap = NULL;
  wasmtime_error_t *error = wasmtime_func_call(
      context, function, args, nargs, results, nresults, &trap);
  if (error != NULL || trap != NULL) fail_wasmtime(name, error, trap);
}

SEXP corrplot_wasm_validate(SEXP module_path_sexp, SEXP json_sexp) {
  const char *module_path = CHAR(STRING_ELT(module_path_sexp, 0));
  const char *json = CHAR(STRING_ELT(json_sexp, 0));
  size_t json_len = strlen(json);
  FILE *file = fopen(module_path, "rb");
  if (file == NULL) Rf_error("cannot open WASM module: %s", module_path);
  if (fseek(file, 0, SEEK_END) != 0) Rf_error("cannot seek WASM module");
  long file_len = ftell(file);
  if (file_len <= 0 || fseek(file, 0, SEEK_SET) != 0) Rf_error("invalid WASM module size");
  uint8_t *bytes = (uint8_t *)malloc((size_t)file_len);
  if (bytes == NULL) Rf_error("out of memory reading WASM module");
  if (fread(bytes, 1, (size_t)file_len, file) != (size_t)file_len) Rf_error("cannot read WASM module");
  fclose(file);

  wasm_engine_t *engine = wasm_engine_new();
  wasmtime_store_t *store = wasmtime_store_new(engine, NULL, NULL);
  wasmtime_context_t *context = wasmtime_store_context(store);
  wasmtime_module_t *module = NULL;
  wasmtime_error_t *error = wasmtime_module_new(engine, bytes, (size_t)file_len, &module);
  free(bytes);
  if (error != NULL) fail_wasmtime("compile WASM module", error, NULL);

  wasmtime_instance_t instance;
  wasm_trap_t *trap = NULL;
  error = wasmtime_instance_new(context, module, NULL, 0, &instance, &trap);
  if (error != NULL || trap != NULL) fail_wasmtime("instantiate WASM module", error, trap);
  wasmtime_module_delete(module);

  wasmtime_extern_t memory_export, alloc_export, free_export, validate_export;
  if (!wasmtime_instance_export_get(context, &instance, "memory", 6, &memory_export) ||
      memory_export.kind != WASMTIME_EXTERN_MEMORY) Rf_error("WASM module must export memory");
  if (!wasmtime_instance_export_get(context, &instance, "corrplot_alloc", 13, &alloc_export) ||
      alloc_export.kind != WASMTIME_EXTERN_FUNC) Rf_error("WASM module must export corrplot_alloc");
  if (!wasmtime_instance_export_get(context, &instance, "corrplot_free", 13, &free_export) ||
      free_export.kind != WASMTIME_EXTERN_FUNC) Rf_error("WASM module must export corrplot_free");
  if (!wasmtime_instance_export_get(context, &instance, "corrplot_validate_json", 22, &validate_export) ||
      validate_export.kind != WASMTIME_EXTERN_FUNC) Rf_error("WASM module must export corrplot_validate_json");

  wasmtime_val_t args[2], results[1];
  args[0].kind = WASMTIME_I32;
  args[0].of.i32 = (int32_t)json_len;
  call_export(context, &alloc_export.of.func, args, 1, results, 1, "call corrplot_alloc");
  uint32_t input_ptr = (uint32_t)results[0].of.i32;
  uint8_t *memory = wasmtime_memory_data(context, &memory_export.of.memory);
  size_t memory_len = wasmtime_memory_data_size(context, &memory_export.of.memory);
  if (input_ptr == 0 || input_ptr > memory_len || json_len > memory_len - input_ptr)
    Rf_error("WASM input buffer is outside linear memory");
  memcpy(memory + input_ptr, json, json_len);

  args[0].kind = WASMTIME_I32; args[0].of.i32 = (int32_t)input_ptr;
  args[1].kind = WASMTIME_I32; args[1].of.i32 = (int32_t)json_len;
  call_export(context, &validate_export.of.func, args, 2, results, 1, "call corrplot_validate_json");
  uint64_t packed = (uint64_t)results[0].of.i64;

  args[0].kind = WASMTIME_I32; args[0].of.i32 = (int32_t)input_ptr;
  args[1].kind = WASMTIME_I32; args[1].of.i32 = (int32_t)json_len;
  call_export(context, &free_export.of.func, args, 2, NULL, 0, "free WASM input");

  uint32_t output_ptr = (uint32_t)(packed & UINT64_C(0xffffffff));
  uint32_t output_len = (uint32_t)(packed >> 32);
  memory = wasmtime_memory_data(context, &memory_export.of.memory);
  memory_len = wasmtime_memory_data_size(context, &memory_export.of.memory);
  if (output_ptr == 0 || output_len == 0 || output_ptr > memory_len || output_len > memory_len - output_ptr)
    Rf_error("WASM output buffer is outside linear memory");

  char *output = (char *)R_alloc((size_t)output_len + 1, 1);
  memcpy(output, memory + output_ptr, output_len);
  output[output_len] = '\0';

  args[0].kind = WASMTIME_I32; args[0].of.i32 = (int32_t)output_ptr;
  args[1].kind = WASMTIME_I32; args[1].of.i32 = (int32_t)output_len;
  call_export(context, &free_export.of.func, args, 2, NULL, 0, "free WASM output");

  wasmtime_store_delete(store);
  wasm_engine_delete(engine);
  return Rf_mkString(output);
}
