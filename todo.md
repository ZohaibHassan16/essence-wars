# To Do

## (D) Review and Update MCP Server

- Bring up to date if needed
- Review `explain_rules()` , so that LLM Agents can play with confidence
- Review and Audit complete Package
- Discuss if AI Hint function needs to be Improved/Enhanced (Test it in a life game first) - does it use Alpha-Beta Depth 6 (it is the best bot we have)

## (F) Review and Update Python Gym

- Look into the changes, update and integrate everything
- review Github Action workflows for Maturin/PyPI for Publishing to Github Packages / PyPI

## (I) Expansion Planning

| Question | Decision |
|----------|----------|
| Scripting vs Pure Rust for cards/keywords/effects | Pure Rust for performance |
| Custom YAML cards | no |
| Bot/Agent compatibility | Accept retuning; keep observation/action space stable |
| Expansion structure | 1 commander + deck per faction per expansion + 20-50 new cards per faction |

## (Z) Create new Documentation Suite

- Each Crate/Python gets its onw Readme.
- Organize `docs/` better
- Target Audience mostly human contributors (AI Developers read the source code)
- Sorted by Game Design, Art Asset Pipeline, ML/AI Infrastructure, ML/AI Research/Experiments, etc.
- One good Main README.md that is top notch and polished (with pictures etc)