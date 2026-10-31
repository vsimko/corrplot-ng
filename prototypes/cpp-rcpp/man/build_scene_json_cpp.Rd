\name{build_scene_json_cpp}
\alias{build_scene_json_cpp}
\title{Build Renderer-Neutral Scene-v1 JSON}
\description{Builds a renderer-neutral scene-v1 JSON representation from labels, a numeric matrix, and optional annotation text.}
\usage{
build_scene_json_cpp(labels, matrix, annotation = "")
}
\arguments{
\item{labels}{Character vector containing the matrix labels.}
\item{matrix}{Numeric square matrix containing the correlation values.}
\item{annotation}{Optional character string included as scene metadata.}
}
\value{A single character string containing the scene-v1 JSON.}
\examples{
labels <- c("a", "b")
correlations <- matrix(c(1, 0.2, 0.2, 1), nrow = 2)
scene <- build_scene_json_cpp(labels, correlations, "example")
stopifnot(is.character(scene), length(scene) == 1L)
}
