% Generated manually for the provisional test scaffold.
\name{corrplot_ng}
\alias{corrplot_ng}
\title{Provisional corrplot-ng interface}
\usage{
corrplot_ng(corr, ...)
}
\arguments{
\item{corr}{A correlation matrix accepted by \code{corrplot::corrplot()}.}
\item{...}{Arguments passed unchanged to \code{corrplot::corrplot()}.}
}
\value{
The result returned by \code{corrplot::corrplot()}.
}
\description{
This is a temporary compatibility adapter. It delegates all work to the
original \pkg{corrplot} package and does not implement a new plotting engine.
Its purpose is to establish an R-facing API and test harness that can later
be redirected to a new implementation.
}
