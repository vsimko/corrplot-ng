\name{corrplot_ng_render_svg}
\alias{corrplot_ng_render_svg}
\title{Render a Scene-V1 Object as SVG}
\description{Renders a scene-v1 object with the shared Rust SVG renderer. The initial R integration supports circle glyph scenes only.}
\usage{
corrplot_ng_render_svg(scene)
}
\arguments{
\item{scene}{A scene-v1 object returned by \code{corrplot_ng(engine = "scene")}.}
}
\value{A character string containing SVG markup.}
\details{
This experimental renderer uses the shared Rust scene parser and SVG renderer. It does not open or modify an R graphics device. The scene object is serialized with \pkg{jsonlite}, which must be installed. Unsupported scene-v1 fields and glyph methods are rejected rather than silently approximated.
}
\seealso{\code{corrplot_ng}, \code{corrplot_ng_normalize_matrix}}
\keyword{hplot}
