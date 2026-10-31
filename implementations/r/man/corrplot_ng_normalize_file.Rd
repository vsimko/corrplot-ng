% Generated manually for the normalized file adapter.
\name{corrplot_ng_normalize_file}
\alias{corrplot_ng_normalize_file}
\title{Normalize CSV, TSV, or normalized-matrix JSON input}
\usage{
corrplot_ng_normalize_file(path, format = c("auto", "csv", "tsv", "json"),
  header = TRUE, row_labels = TRUE)
}
\arguments{
\item{path}{Path to a UTF-8 input file.}
\item{format}{Input format: \code{auto} (by extension), \code{csv}, \code{tsv}, or \code{json}.}
\item{header}{Whether CSV/TSV contains column labels.}
\item{row_labels}{Whether the first CSV/TSV column contains row labels.}
}
\value{
A list with class \code{corrplot_ng_normalized_matrix_v1}. JSON input must already conform to normalized-matrix v1.
}
\description{
Reads an already-computed square numeric matrix from CSV or TSV using the publication-config layout, or reads an existing normalized-matrix v1 JSON document. When both row and column labels are supplied they must match exactly and in order. A single supplied label axis is reused for both axes; absent labels are generated as \code{V1} through \code{Vn}. Missing cells remain \code{NULL}, numeric zero remains zero, and non-numeric, NaN, infinite, duplicate, or dimensionally invalid inputs are rejected.
}
