% Generated manually for the provisional R adapter.
\name{corrRect.hclust}
\alias{corrRect.hclust}
\title{Add rectangles for hierarchical clusters}
\usage{
corrRect.hclust(corr, k = 2, col = "black", lwd = 2,
  method = c("complete", "ward", "ward.D", "ward.D2", "single",
    "average", "mcquitty", "median", "centroid"))
}
\arguments{
\item{corr}{Finite square numeric correlation matrix.}
\item{k}{Integer number of clusters, between 1 and the matrix dimension.}
\item{col}{R graphics color used for rectangle borders.}
\item{lwd}{Border line width between 0.1 and 12.}
\item{method}{Hierarchical clustering method accepted by the corrplot 0.95 compatibility baseline.}
}
\value{
Invisibly returns \code{NULL}, matching the legacy drawing helper.
}
\description{
Clusters \code{1 - corr} with \code{stats::hclust()}, cuts the dendrogram,
and draws one rectangle per cluster in leaf order. The corresponding
cluster-region requests use the shared scene-v1 input shape.
}
