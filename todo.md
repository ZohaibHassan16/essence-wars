# To Do

## (A) Deck Builder

- Custom Deck Builder
- Choose Faction -> Choose Commander
- Choose Cards from Faction + Neutral Cards 
- Save as Deck, add Description
- Be able to play with custom decks
- Cannot edit or overwrite the prebuilt 12 Starter Decks, only custom decks

## (B) Update Lore

Update `lore.md` with the new direction for Argentum, Symbiote and the Free Walkers in our Art Direction `docs/art-direction.md` and the prompts(Artwork). Free Walkers not using guns, Symbiote not being bio punk engineers but naturalists and druids etc. Make sure the whole Document is in sync, including `docs/essence-wars-design.md` Faction Descriptions.

## (C) Update and Enhance Diagnosis and Analysis Capabilities of Spectator Mode

Update and Enhance the Diagnosis/Analysis Screen in Spectator Mode, using the now fully enhanced binary libraries we have at our hand, with plots, graphs, tables, statistics, probabilities etc.

## (D) Review and Update MCP Server

- Bring up to date
- Discuss if a needed `explain_rules()` function should be added, that concisely explain all rules, so that LLM Agents can play with confidence
- Review and Audit complete Package
- Improve/Enhance AI Hint function

## (F) Review and Update Python Gym

- Look into the changes, update and integrate everything
- Create new, updated Github Action workflows - Fresh workflows with current best practices for Maturin/PyPI for Publishing to Github Packages / PyPI

## (G) Review if Asset Pipeline is still in sync

Search and review all Asset related Scripts, Data, Prompts, Artwork, Documention etc, and bring them up to date and in sync.

### (H) Tutorials

- Update Human vs AI Tutorial
- Create Spectator Mode Tutorial 
- Create Deck Builder Tutorial

 Hello Claude.

  I want to tackle Phase 9, Updating the Tutorial/Onboarding Experience from @docs/web-client-design.md . Let us please discuss this.

  There already exists a Tutorial but it needs to be reworked, the ai enemy commander is not following the steps (e.g. not playing a creature etc), and the highlight / shading is too dark, I can not see much outside of the highlighted area, which is not really helpful, as I can not see where I need to click. I think it should be following a different style, maybe a light glow on the area where we want to focus the player on, not darkening the rest of the board? E.g. first glow the card that needs to be clicked, than the glow the goal that needs to be clicked. Also, Glow on the resources and the commanders as they are presented, and so on. It can be blocking so that the new player not accidently clicks wrong, only the actual glowing/highlighted intended next action, so that the tutorial does not break.

  It must not be a complete full game, that would enduce fatigue and frustrate new players, it can remain open ended, ending with the AI Hint system for players that might need more help (which basically auto plays for you if you want, you can just click it).