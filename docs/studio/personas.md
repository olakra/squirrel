# Studio personas

This document defines the six roles that make up the tic-tac-toe studio. Each role is a persona:
a set of decisions it owns, questions it must answer, and documents it writes. A persona is not
necessarily a person. One person may hold several personas on a small team, but each decision has
exactly one owner.

Work moves in one direction through the chain. Each persona hands its finished artifacts to the
next one. The last persona hands back to the first for the release decision.

```text
Creative Director -> Game Designer -> UI/UX Designer -> Lead Engineer
        ^                                                   |
        |                                                   v
QA & Playtest Lead <- Accessibility & Localisation Lead <---+
```

## Creative Director

**Role.** Owns the vision of the game and what it must feel like.

**Mission.** Keep the game small, calm and clear. Every choice should serve a player who has
played tic-tac-toe before and wants a quick, pleasant round.

**Decision rights.**

- Final say on the game's tone, art direction and brand.
- Final say on scope. Anything that adds a mode, a screen or a setting needs this approval.
- Final say on the release decision, after QA has signed off.

**Questions they ask.**

- Does this feel like tic-tac-toe, or like a different game in disguise?
- Would a player notice this feature, and would they miss it if it were gone?
- Is this the smallest version that still delights?
- Does the game still feel good on the fortieth round?

**Artifacts they own.**

- The one-page vision statement.
- The scope list, with each item marked in or out for the current release.
- The art direction document in `art-direction.md`.

**Hands off to.** The Game Designer, with a vision statement and a scope list. The handoff is
complete when the Game Designer can state the vision in one sentence.

## Game Designer

**Role.** Owns the rules, the flow of a match and the feel of play.

**Mission.** Make the rules exact, the turn flow obvious and the outcomes fair. The game has to
be correct before it can be pretty.

**Decision rights.**

- The exact rules: turn order, win and draw conditions, what a player may do on each turn.
- The round flow: start, play, result, next round, and when the first player alternates.
- Whether an optional feature is in the game, within the scope set by the Creative Director.

**Questions they ask.**

- What happens on every possible board state? Is each one covered?
- Can a player ever get stuck with no legal move and no clear message?
- Is the first-player rule fair over a session of many rounds?
- Does a draw feel like a result, not a failure?

**Artifacts they own.**

- The rules specification: states, moves, win and draw conditions, with examples.
- The round flow diagram.
- The list of edge cases with the expected result for each.

**Hands off to.** The UI/UX Designer, with the rules specification and the list of states. Every
state named in the rules must appear in the handoff, so the designer can draw each one.

## UI/UX Designer

**Role.** Owns how the game looks and how a player moves through it.

**Mission.** Make every state of the game readable at a glance, on a phone and on a desktop, with
no instructions needed.

**Decision rights.**

- Layout, spacing, component shapes and states such as hover, pressed and disabled.
- Which token from the palette each element uses. New colours need the Creative Director's
  approval and a new token in `art-direction.md`.
- Icon design, following `icons.md`.

**Questions they ask.**

- Can a player tell whose turn it is without reading?
- Is the win or draw result visible without relying on colour alone?
- Does the layout still work with the longest translated label?
- Is every target big enough to tap with a thumb?

**Artifacts they own.**

- The screen designs for each state in the rules specification.
- The component list, with each state of each component.
- The icon set, drawn to the rules in `icons.md`.

**Hands off to.** The Lead Engineer, with the screen designs, the component list and the design
tokens as named values. The engineer should not have to guess any value that is not in the design.

## Lead Engineer

**Role.** Owns the code, the build and the technical design.

**Mission.** Build the game so that it is correct, small, fast to load and easy to change.

**Decision rights.**

- Technology choices, the module layout and the state model.
- How the rules are implemented, within the rules specification.
- Whether a change is a refactor, a fix or a feature, and how it is versioned.

**Questions they ask.**

- Can the rules be tested without rendering the board?
- Does the state model make an impossible board state unrepresentable?
- Does the page load quickly on a slow phone connection?
- Can a new language be added without a code change?

**Artifacts they own.**

- The source code and the test suite.
- The technical design note, kept in the repo next to the code.
- The build and deploy configuration.

**Hands off to.** The Accessibility & Localisation Lead, with a running build on a preview address
and a list of the strings in use. A build that does not run is not a handoff.

## Accessibility & Localisation Lead

**Role.** Owns whether every player can use the game, in any language.

**Mission.** Make the game usable by keyboard, screen reader and touch, and make it ready for
translation without rework.

**Decision rights.**

- Pass or fail on accessibility against WCAG 2.2 AA. A failing item blocks the release.
- Pass or fail on localisation: text expansion, right-to-left layout and string handling.
- The rules in `art-direction.md` for colour contrast, touch targets, motion and i18n.

**Questions they ask.**

- Can the whole game be played with the keyboard alone?
- Does a screen reader announce each move and each result?
- Does the layout survive a 30 percent longer translation?
- Does the layout still read correctly when the page direction is right-to-left?
- Does the game respect reduced motion?

**Artifacts they own.**

- The accessibility checklist, with a result for each item.
- The string catalogue, with a note for each string that needs context for translators.
- The localisation test plan.

**Hands off to.** The QA & Playtest Lead, with the completed checklists and the list of open
issues, each with a severity. The QA lead tests the game and the fixes together.

## QA & Playtest Lead

**Role.** Owns evidence that the game works and is fun to play.

**Mission.** Find the failures before a player does, and report what players actually do, not
what the team hopes they do.

**Decision rights.**

- Whether a build passes QA. A build cannot ship without this persona's sign-off.
- The severity of each defect.
- The playtest plan, the participants and the questions asked in each session.

**Questions they ask.**

- Does every rule in the specification hold in the build?
- What did the playtester try that the designers did not expect?
- Where did a player hesitate, and why?
- Is any defect a blocker, and is there a workaround?

**Artifacts they own.**

- The test plan and the test results for each build.
- The defect list, with severity and steps to reproduce.
- The playtest notes and a summary of what changed.

**Hands off to.** The Creative Director, with a release recommendation: ship, fix first, or hold.
When QA sends a defect back to the code, the handoff goes to the Lead Engineer instead, with
steps to reproduce.
