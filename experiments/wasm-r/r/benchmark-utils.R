# Shared repeated-batch benchmark helper for the R runtime comparison.
# Each sample is a separately timed batch in the same warmed process.
# The 95% CI is a Student-t interval over batch-level per-call means (n=10).
benchmark_repeated <- function(invoke) {
  sample_count <- as.integer(Sys.getenv("CORRPLOT_BENCH_SAMPLES", "10"))
  target_seconds <- as.numeric(Sys.getenv("CORRPLOT_BENCH_TARGET_SECONDS", "1"))
  if (is.na(sample_count) || sample_count < 2L) stop("CORRPLOT_BENCH_SAMPLES must be >= 2")
  if (!is.finite(target_seconds) || target_seconds <= 0) stop("CORRPLOT_BENCH_TARGET_SECONDS must be > 0")

  invisible(replicate(5L, invoke(), simplify = FALSE))
  pilot_calls <- 20L
  started <- proc.time()[["elapsed"]]
  for (i in seq_len(pilot_calls)) response <- invoke()
  pilot_seconds <- proc.time()[["elapsed"]] - started
  calls_per_sample <- max(1L, min(2000000L,
    as.integer(ceiling(target_seconds / max(pilot_seconds / pilot_calls, 1e-6)))))

  sample_elapsed_ms <- numeric(sample_count)
  for (sample in seq_len(sample_count)) {
    started <- proc.time()[["elapsed"]]
    for (i in seq_len(calls_per_sample)) response <- invoke()
    sample_elapsed_ms[[sample]] <- (proc.time()[["elapsed"]] - started) * 1000
  }

  per_call <- sample_elapsed_ms / calls_per_sample
  mean_ms <- mean(per_call)
  sd_ms <- stats::sd(per_call)
  t_critical <- stats::qt(0.975, df = sample_count - 1L)
  half_width <- t_critical * sd_ms / sqrt(sample_count)

  list(
    calls = calls_per_sample * sample_count,
    callsPerSample = calls_per_sample,
    sampleCount = sample_count,
    targetSecondsPerSample = target_seconds,
    elapsedMs = sum(sample_elapsed_ms),
    perCallMs = mean_ms,
    meanPerCallMs = mean_ms,
    sdPerCallMs = sd_ms,
    ci95LowerMs = max(0, mean_ms - half_width),
    ci95UpperMs = mean_ms + half_width,
    samplesPerCallMs = unname(per_call)
  )
}
