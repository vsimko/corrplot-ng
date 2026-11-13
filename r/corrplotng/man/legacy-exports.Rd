\name{legacy-exports}
\alias{corrplot}
\alias{corrplot.mixed}
\alias{corrMatOrder}
\title{Legacy corrplot 0.95 API entry points}
\description{
These names are re-exported from the pinned upstream \pkg{corrplot} 0.95
package so scripts can use the original function names after attaching
\pkg{corrplotng}. Their formal arguments, defaults, and behavior are the
upstream contract while the legacy renderer remains the compatibility
fallback.
}
\details{
\code{corrplot} draws a correlation or general numeric matrix.
\code{corrplot.mixed} draws different methods in the upper and lower
triangles. \code{corrMatOrder} returns a variable ordering for a matrix.
The current re-exports intentionally delegate to upstream; they do not imply
that the Rust renderer implements the complete legacy feature set.

See the pinned compatibility inventory and audit in the repository
documentation for the full argument contract and migration status.
}
\seealso{
\code{\link{corrplot_ng}}, \code{\link{corrplot_ng_mixed}},
\code{\link{corrMatOrder}}
}
