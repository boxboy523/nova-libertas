# Collaboration rules

## Editing authorization

- Default to read-only analysis and discussion.
- Do not modify project files unless the user explicitly asks for a modification.
- A statement of intent, an observation that something should change, or agreement with a proposed direction is not authorization to edit.
- Before editing, explain the relevant current behavior, the proposed change, and its expected interaction with the existing system. Wait for an explicit request to implement it.
- When the user explicitly requests an edit, that request authorizes only the stated scope. Discuss materially broader changes before making them.

## Change explanations

- After every requested modification, explain exactly what was implemented.
- Describe how the changed code participates in the existing architecture, including relevant callers, resources, events, components, systems, schedules, and data or control flow.
- Explain behavior before and after the change, important ordering or lifecycle constraints, and any affected invariants or tradeoffs.
- Report the files and important symbols changed, the validation performed, and any remaining uncertainty or risk.
- Provide enough concrete detail that the developer can keep the complete system flow in mind. Do not reduce the handoff to a terse list of edited files.

## Documentation boundaries

- Keep this file focused on collaboration rules, ownership, workflow constraints, and validation expectations.
- Store durable architecture, behavior, data-flow, debugging, and design knowledge in `ai-docs.org` rather than expanding this file into a project knowledge dump.
- Keep documentation changes subject to the same explicit-edit requirement as source changes.
