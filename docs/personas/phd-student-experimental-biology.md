# Persona: PhD Student in Experimental Biology

## Context

The user analyses experimental data in RStudio. A colleague recommends the R package `corrplot`. While looking for it, the user also finds `corrplot-ng`.

## Main goal

Create a clear correlation plot for a scientific paper. The user wants to spend time on the research, not on the plotting tool.

## Skills and tools

- Uses R and RStudio for data analysis.
- Can edit and run R scripts.
- Does not want to manage compilers, language toolchains, or system libraries.
- May use a university-managed computer with limited permission to install system software.

## Needs

- Install the package with the usual R workflow.
- Run it from RStudio with a short, clear example.
- Reuse existing `corrplot()` calls where possible.
- Control labels, colours, layout, and annotations.
- Get stable and reproducible output for reports and papers.
- Find clear examples and troubleshooting guidance.

## Adoption barriers

- Installation requires manual setup of a compiler or another language toolchain.
- Existing scripts stop working or produce unexpected plots.
- The package has unclear benefits compared with `corrplot`.
- Documentation does not show how to solve a common research task.
- The package does not provide clear versioning or citation information.

## Success criteria

The user can install the package, try it on their own correlation matrix, and create a usable plot with few or no changes to an existing R workflow.

## Product implications

- Treat R as a first-class interface.
- Keep installation simple on supported platforms.
- Test legacy behaviour and visual output.
- Provide short, copyable examples based on realistic data.
- Explain limitations and compatibility status clearly.
- Do not require users to know which language implements the shared core.

## Key question

**Does `corrplot-ng` make this user's work easier or more reliable than the existing `corrplot` package?**

A new name alone is not a reason to switch. The project must show a clear benefit without adding avoidable work.
