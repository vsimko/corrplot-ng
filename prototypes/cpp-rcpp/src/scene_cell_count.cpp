#include <Rcpp.h>

// [[Rcpp::export]]
int scene_cell_count(int n_labels) {
  if (n_labels < 0) {
    Rcpp::stop("n_labels must be non-negative");
  }
  return n_labels * n_labels;
}
