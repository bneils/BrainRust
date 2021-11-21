To run a Brainfuck program,

./brainfuck file.bf

You can pass multiple file paths to run more than one program.

An optimization to make this interpreter faster is run-length opcode encoding.

For example, the instructions "++-+." will be simplified as "Add(2), Print"
