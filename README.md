# Anura

My attempt to learn rust, by making another chess engine in it.

Anura was chosen as the name because it's the biological order of frogs.

Current notes:
- Anura 1.0 will be released when it hits or surpasses 3000 Elo CCRL Blitz, + 30 or so for some margin of error.
- Goals:
    - Multilayer value inference
    - Threat Inputs (value and policy)
    - SEE outputs (policy)
    - generally increase HLs overall
    - anything else I can think of

This has been quite the project so far and it wouldn't be possible without the programs, resources, and people who've helped me out each step of the way:

Programs I've used:
- Bullet https://github.com/jw1912/bullet
    - used as my network trainer with a modified example
    - used for policy as well with an adjusted version of...
- montytrain https://github.com/official-monty/montytrain/
    - used for training policy
- montyformat https://github.com/official-monty/montyformat
    - policy data format of choice

Resources I've used:
- Monty https://github.com/official-monty/Monty/
    - incredible engine made by some dope people (who are willing to tell me their knowledge)
    - tunable system is from them as well, they do it really neatly and I liked it,
- Jackal https://github.com/TomaszJaworski777/Jackal/
    - direct competitor in the MCTS EAS business (which I'm not even really a part of yet) but has been helpful
- Voidstar https://github.com/Ciekce/voidstar/
    - very simple engine but very helpful with my movegen struggles, and helped me understand the concepts

People
- Jamie Whiting (JW) https://github.com/jw1912/
    - ~~More random C++ things~~
    - Very good at dealing with my antics
    - Very willing to share information
- Viren https://github.com/Viren6
    - "Viren's great big gainer"
    - Also puts up with me
    - Helps with information
    - Hardware for montytest
- Tomasz Jaworski (Snekker) https://github.com/TomaszJaworski777
    - epic individual
    - very dope

THIS IS NOT A COMPLETE LIST, there are certainly people and programs that i've forgotten, but I am thanksful for each and every one of you who've helped me with this project!!
