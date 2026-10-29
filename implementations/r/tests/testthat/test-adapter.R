test_that("the provisional adapter delegates to the legacy corrplot package", {
  corr <- matrix(
    c(1, 0.6, 0.6, 1),
    nrow = 2,
    dimnames = list(c("A", "B"), c("A", "B"))
  )

  for (method in c("circle", "square", "ellipse", "number", "shade", "color", "pie")) {
    path <- tempfile(fileext = ".pdf")
    grDevices::pdf(path)
    tryCatch(
      expect_no_error(
        corrplot_ng(
          corr,
          method = method,
          type = "full",
          tl.pos = "n",
          cl.pos = "n"
        )
      ),
      finally = {
        grDevices::dev.off()
        unlink(path)
      }
    )
  }
})

test_that("the adapter forwards common plotting arguments", {
  corr <- matrix(
    c(1, -0.4, -0.4, 1),
    nrow = 2,
    dimnames = list(c("A", "B"), c("A", "B"))
  )
  path <- tempfile(fileext = ".pdf")
  grDevices::pdf(path)
  tryCatch(
    expect_no_error(
      corrplot_ng(
        corr,
        method = "color",
        order = "original",
        addCoef.col = "black",
        tl.col = "black"
      )
    ),
    finally = {
      grDevices::dev.off()
      unlink(path)
    }
  )
})
