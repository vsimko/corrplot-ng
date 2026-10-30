% Generated for corrplot-ng plotmath label compatibility.
\name{corrplot_ng_parse_label}
\alias{corrplot_ng_parse_label}
\title{Parse legacy corrplot plotmath labels}
\usage{
corrplot_ng_parse_label(label)
}
\arguments{
\item{label}{One non-missing character string.}
}
\value{An R expression for labels prefixed with `:`, `=`, or `$` (without the space after the prefix); otherwise the original character string.}
\description{Parses the plotmath expression syntax used by corrplot 0.95. Parsing does not evaluate the expression.}
