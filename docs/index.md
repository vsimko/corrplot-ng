---
okf_version: "0.2"
type: Documentation Index
title: corrplot-ng documentation
description: Index and editing guidelines for the OKF-based project documentation.
tags:
  - documentation
  - okf
  - writing-guidelines
status: active
---

# corrplot-ng documentation

This directory contains project documentation in the Open Knowledge Format (OKF).

Use this file as the starting point. Keep all repository documentation in English.

## Project at a glance

The main requirements are:

1. Keep the original R package useful and stable.
2. Preserve legacy R behavior in the new project, with compatibility claims backed by tests.
3. Use a shared core and clear interfaces for multiple programming languages.
4. Keep shared plot behavior consistent across languages.
5. Support the common correlation-plot methods from the original package.
6. Document features, defaults, limitations, and examples.
7. Keep the design small enough to maintain with a few hours of work per week and AI assistance.
8. Provide a migration aid and citation support as planned follow-up work.

See [top-level requirements](requirements.md) for priorities and checks. These are project goals, not claims about the current implementation.

## Documents

- [Top-level requirements](requirements.md): main product and engineering requirements, priorities, and acceptance checks.
- [Proposed architecture](architecture.md): components, dependency direction, language interfaces, shared core, and rendering backends.
- [Future vision](future-vision.md): project goals, possible development steps, and open questions.
- [Catalog of diagrams and visualizations](diagram-catalog.md): compatibility targets, shared display options, and ideas for future visualizations.

## Editing guidelines

### 1. Keep the text simple

Follow the principles of ASD-STE100 where practical.

- Use short sentences.
- Prefer common words.
- Use one main idea in each sentence.
- Use active voice when possible.
- Use direct instructions.
- Explain a technical term when readers may not know it.
- Avoid idioms, jokes, and vague phrases.
- Use the same term for the same concept throughout the documentation.

### 2. Keep all content in English

Write headings, body text, metadata, examples, comments, and file names in English. Keep code identifiers unchanged when they are part of an API or example.

### 3. Use OKF-style Markdown

Each knowledge document should use Markdown and YAML frontmatter.

- Add a clear title.
- Add a short description.
- Add useful tags.
- Add a type when the document type is clear.
- Add a status when the document is a draft or proposal.
- Keep metadata values short and factual.
- Do not add metadata fields that have no clear use.

Follow the format used by the existing documents in this directory. If the project adopts a stricter OKF schema later, update the documents to match it.

### 4. Separate facts from plans

Use clear wording to show the status of each statement.

- **Current:** verified in the repository or tests.
- **Planned:** an agreed goal that is not implemented yet.
- **Proposed:** an idea that needs review.
- **Open question:** a decision that the project has not made.

Do not describe a planned or proposed feature as implemented. Do not promise a release date unless the project has agreed to it.

### 5. Keep documents focused

- Give each file one main purpose.
- Put project-wide goals in the future vision document.
- Put plot types and visualization ideas in the diagram catalog.
- Keep general writing rules in this index.
- Link to related documents instead of copying large sections.
- Use relative links for files in this directory.

### 6. Make changes easy to review

- Use clear headings and short lists.
- Use tables only when they make comparisons easier.
- Keep each change focused on one topic.
- Check all links after moving or renaming a file.
- Update this index when you add, remove, or rename a document.
- Prefer small changes that reviewers can understand quickly.

### 7. Record decisions and uncertainty

When a design is not final, say so. Record the question and the reason it matters. When the team makes a decision, update the relevant document and remove outdated statements.

### 8. Review before commit

Before you commit a documentation change, check that:

- The text is in English.
- The sentences are short and clear.
- The frontmatter is valid YAML.
- The status of each feature is clear.
- The links work.
- The document does not conflict with the current code or tests.
- This index reflects the current document set.

## Suggested document template

Use this template for a new planning or knowledge document:

```yaml
---
type: Document Type
title: Clear document title
description: One sentence that explains the purpose.
tags:
  - relevant-topic
status: draft
---
```

After the frontmatter, add a short purpose section. Use headings that describe the content. Mark proposals and open questions clearly.

## About OKF

The project uses OKF-style Markdown and YAML frontmatter to make knowledge clear and structured. These are project editing rules. They do not claim that every field or rule is part of a formal OKF specification.
