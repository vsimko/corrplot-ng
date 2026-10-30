% Generated manually for the provisional adapter.
\name{corrplot_ng}
\alias{corrplot_ng}
\title{corrplot-ng compatibility adapter}
\usage{
corrplot_ng(corr, ..., engine = c("legacy", "scene"))
}
\arguments{
\item{corr}{A numeric matrix. The legacy engine accepts matrix forms supported by \code{corrplot::corrplot()}; the scene engine requires a non-empty square correlation matrix.}
\item{...}{Arguments forwarded to \code{corrplot::corrplot()} by the legacy engine. The scene engine accepts only an optional \code{annotation} string and rejects other arguments.}
\item{engine}{Rendering path: \code{"legacy"} (default) or experimental \code{"scene"} (returns a scene-v1 object without drawing).}
}
\value{
For \code{engine = "legacy"}, the result returned by \code{corrplot::corrplot()}. For \code{engine = "scene"}, a \code{corrplot_ng_scene} list conforming to scene-v1.
}
\description{
The default legacy engine delegates to the original \pkg{corrplot} implementation. The experimental scene engine prepares the supported matrix-and-label subset as a renderer-neutral scene-v1 object; it does not draw on an R graphics device.
}
