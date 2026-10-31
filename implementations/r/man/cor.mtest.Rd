% Generated manually for the pairwise correlation test helper.
\name{cor.mtest}
\alias{cor.mtest}
\title{Pairwise correlation tests and confidence intervals}
\usage{
cor.mtest(mat, ...)
}
\arguments{
\item{mat}{Numeric matrix or data frame. Rows are observations and columns are features.}
\item{...}{Additional arguments forwarded to \code{stats::cor.test()}, including \code{method}, \code{alternative}, \code{exact}, \code{continuity}, and \code{conf.level}.}
}
\value{
A list containing three symmetric square matrices: \code{p} (p-values), \code{lowCI} (lower confidence limits), and \code{uppCI} (upper confidence limits). Matrix dimnames are copied from input feature names. The diagonal follows corrplot 0.95 conventions: p-values are zero and confidence limits are one.
}
\description{
Performs pairwise correlation tests using complete observations independently for each feature pair. Pearson confidence intervals are copied from \code{stats::cor.test()}; methods that do not return confidence intervals leave off-diagonal confidence limits missing. Pairs with fewer than three complete observations or a constant feature return missing p-values and confidence limits. Non-numeric columns and infinite input values are rejected.
}
\seealso{
\code{stats::cor.test}
}
