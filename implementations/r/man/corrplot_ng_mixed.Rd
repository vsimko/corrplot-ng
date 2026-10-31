% Generated manually for the provisional adapter.
\name{corrplot_ng_mixed}
\alias{corrplot_ng_mixed}
\title{Mixed legacy corrplot compatibility adapter}
\usage{
corrplot_ng_mixed(corr, ..., engine = "legacy")
}
\arguments{
\item{corr}{A correlation matrix accepted by \code{corrplot::corrplot.mixed()}.}
\item{...}{Arguments forwarded unchanged to \code{corrplot::corrplot.mixed()}.}
\item{engine}{Only \code{"legacy"} is currently supported. Mixed scene rendering is not implemented.}
}
\value{
The invisible list returned by \code{corrplot::corrplot.mixed()}, containing the reordered correlation matrix and combined \code{corrPos} data frame.
}
\description{
A thin compatibility wrapper around the upstream mixed correlation plot function. It preserves the upstream return structure and visibility.
}
