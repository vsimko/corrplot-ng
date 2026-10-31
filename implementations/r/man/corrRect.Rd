% Generated manually for the provisional R adapter.
\name{corrRect}
\alias{corrRect}
\title{Add cluster rectangles to a legacy corrplot}
\usage{
corrRect(corrRes = NULL, index = NULL, name = NULL, namesMat = NULL,
  col = "black", lwd = 2, ...)
}
\arguments{
\item{corrRes}{Invisible result returned by \code{corrplot_ng()} or \code{corrplot_ng_mixed()}.}
\item{index}{Increasing one-based boundary positions. Each adjacent pair defines one inclusive cluster range.}
\item{name}{Ordered variable names defining the same boundary ranges as \code{index}.}
\item{namesMat}{A four-element character vector or a character matrix with columns \code{xleft}, \code{ybottom}, \code{xright}, and \code{ytop}; each corner is resolved through \code{corrRes$corrPos}.}
\item{col}{R graphics color used for rectangle borders.}
\item{lwd}{Border line width between 0.1 and 12.}
\item{...}{Additional arguments passed to \code{graphics::segments()} or \code{graphics::rect()}.}
}
\value{
Invisibly returns \code{corrRes} unchanged.
}
\description{
Draws cluster rectangles on the caller's active graphics device. Index and
name ranges also map to the shared scene-v1 cluster-region request shape.
Coordinate-pair rectangles support differing row and column names.
}
