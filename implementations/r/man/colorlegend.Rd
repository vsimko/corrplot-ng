\name{colorlegend}
\alias{colorlegend}
\title{Draw a standalone color legend}
\usage{
colorlegend(colbar, labels, at = NULL, xlim = c(0, 1), ylim = c(0, 1),
  vertical = TRUE, ratio.colbar = 0.4, lim.segment = "auto",
  align = c("c", "l", "r"), addlabels = TRUE, ...)
}
\arguments{
\item{colbar}{Non-empty character vector of colors for the bar.}
\item{labels}{Numeric or character labels to draw.}
\item{at}{Optional label positions normalized to [0, 1].}
\item{xlim, ylim}{Finite plot-coordinate limits.}
\item{vertical}{Whether to draw a vertical bar.}
\item{ratio.colbar}{Fraction of the legend dimension occupied by the bar.}
\item{lim.segment}{Two normalized tick-span coordinates, or NULL/auto.}
\item{align}{Vertical label alignment: left, center, or right.}
\item{addlabels}{Whether to draw labels.}
\item{...}{Graphical arguments forwarded to graphics::text().}
}
\value{Invisibly returns NULL; called for its graphics side effect.}
\description{Draws a standalone color legend on the active base graphics device.}
\details{Compatible with the corrplot 0.95 colorlegend API and drawing conventions. The caller must set up the plotting region before calling this function.}
