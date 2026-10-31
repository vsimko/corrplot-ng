\name{corrplot_ng_diagnostic}
\alias{corrplot_ng_diagnostic}
\title{Read a structured corrplot-ng diagnostic}
\usage{
corrplot_ng_diagnostic(condition)
}
\arguments{
\item{condition}{A condition caught from a corrplot-ng operation.}
}
\value{
A language-neutral diagnostic-v1 list, or \code{NULL} when the condition
is not a corrplot-ng diagnostic.
}
\description{
Read stable diagnostic fields from a structured corrplot-ng error or warning
condition. The returned record contains a schema version, code, severity,
stage, message, and optional path or zero-based matrix coordinates.
}
