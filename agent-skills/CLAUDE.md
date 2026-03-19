# Agent skills

This is a repository of skills that we use to perform structured development tasks using claude code. 

The workflow is exposed to the Agent and the user through three skills listed below

- dev-strategy: planing skill 
- dev-contribute: development skill
- design-contribute: system/software design skill

This three skills in turn referenc two supporting skills: `contribution-system` and 
`design-principles` that encode the user preferences in terms of how the context should be preserved and 
how to do design / planning and implementation of software

# Context to be aware of : 
- The decision log artifacts ( see `decision-log-*.yaml` in `contribution-system/assets/templates/` ) is meant to be 
parsed and used by diffviz-cli. Therefore, it cannot be changed unless the code of diffviz-cli is changed as well.

