library(corrplotngcppspike)

stopifnot(identical(scene_cell_count(3L), 9L))
stopifnot(identical(scene_cell_count(0L), 0L))

message("C++/Rcpp integration smoke test passed")
