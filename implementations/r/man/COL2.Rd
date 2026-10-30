% Generated for corrplot-ng legacy palette compatibility.
\name{COL2}
\alias{COL2}
\title{Get legacy diverging colors}
\usage{
COL2(diverging = c("RdBu", "BrBG", "PiYG", "PRGn", "PuOr", "RdYlBu"),
  n = 200)
}
\arguments{
\item{diverging}{Diverging palette family.}
\item{n}{Positive integer number of colors to return.}
}
\value{A character vector of hexadecimal colors.}
\description{Returns the corrplot 0.95 diverging palette for the requested family, retaining its white neutral anchor and interpolating to \code{n} colors.}
\seealso{\code{COL1}, \code{grDevices::colorRampPalette}}
