---
title: "Getting Started with checkmate"
description: "A guided setup checklist for your first checkmate workspace"
tags: [starter, onboarding]
---

## Understand the model

- [ ] Read the overview
      checkmate is a desktop app for reusable operational checklists and recorded runs. Templates and run history are stored as plain markdown files on your machine, and git is optional.
- [ ] Use the default path for now
      On first launch, checkmate creates a default data folder automatically. You do not need to open Settings before you can start using the app.
- [ ] Treat this guide as a real checklist
      This file is not special. It is a normal checklist you can run, edit, repurpose, or delete later.

## Know what you can do from this screen

- [ ] Review how checklist content works
      Each step can have a short description and an optional code snippet. This guide uses the same format your own checklists will use.

  ```markdown
  ## Example Section

  - [ ] Example step
        Add context or paste a command, config fragment, or note here
  ```

- [ ] Notice that steps can be edited inline
      If a step needs better wording, use the edit control on the item. That is useful later when you refine your own operational checklists.
- [ ] Understand what happens without git
      checkmate works in a normal folder. Saving checklists and runs still works even if the folder is not a git repository.

## Decide how much setup you actually need

- [ ] Start simple unless you already know you need a custom location
      The fastest path is to keep the default data folder and create your first checklist there.
- [ ] Leave git for later unless version history matters right away
      Auto-commit stays off until your active data folder is a real git repository. You can enable git after you know the basic workflow fits.
- [ ] Remember the upgrade path
      If you later want a custom folder, git history, or GitHub sync, you can add those without changing your checklist workflow.

## Leave this guide only when you are ready for the next step

- [ ] Choose your next action and then leave this checklist
      Recommended first move: click the plus icon in the top bar and make your first checklist in the default location. If you want a custom folder instead, open Settings and point checkmate at that folder.
      If you choose a custom folder with git backing, initialize that folder as a git repository first, then select it in Settings. After you create your first checklist there, enable auto-commit if you want automatic commits.
