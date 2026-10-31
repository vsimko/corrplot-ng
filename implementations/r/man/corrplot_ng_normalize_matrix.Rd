% Generated manually for the normalized matrix input adapter.
\name{corrplot_ng_normalize_matrix}
\alias{corrplot_ng_normalize_matrix}
\title{Normalize an R matrix or data frame}
\usage{
corrplot_ng_normalize_matrix(x, metadata = NULL)
}
\arguments{
\item{x}{A square numeric matrix or a data frame containing only integer or double columns. A data frame is interpreted as an already-computed matrix, not raw observations.}
\item{metadata}{Optional named list of JSON-compatible source metadata.}
}
\value{
A list with class \code{corrplot_ng_normalized_matrix_v1} conforming to normalized-matrix v1. The object contains schema version, kind, labels, nested values, source metadata, and zero-based source indices. Missing R \code{NA} values are represented by \code{NULL}; zero remains numeric zero.
}
\description{
The adapter normalizes an already-computed square matrix without calculating correlations. If both row and column names are supplied, they must match exactly. If only one axis has names, those labels are used for both axes. If neither has names, labels \code{V1} through \code{Vn} are generated. Unicode labels are preserved. Empty or duplicate names, non-square inputs, non-numeric columns, \code{NaN}, and infinite values are rejected.
}
