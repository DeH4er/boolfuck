fn boolfuck(code: &str, input: Vec<u8>) -> Vec<u8> {
    use interpreter::*;
    use utils::*;
    use parser::*;

    let instructions = parse(code);
    let input = from_bytes(&input);
    let mut interpreter = Interpreter::new(instructions, input);
    interpreter.interpret();
    let output = interpreter.get_output();
    to_bytes(output)
}

#[derive (PartialEq, Debug)]
pub enum Instruction {
    Flip,
    Read,
    Write,
    MoveLeft,
    MoveRight,
    SkipRight,
    SkipLeft
}

#[derive (PartialEq, Copy, Clone, Debug)]
pub enum Bit {
    Zero,
    One
}

impl Bit {
    fn flip(&self) -> Bit {
        match self {
            Self::Zero => Self::One,
            Self::One => Self::Zero
        }
    }
}

mod utils {
    use super::*;

    pub fn from_bytes(v: &[u8]) -> Vec<Bit> {
        v
            .iter()
            .flat_map(|num| bits_from_u8(*num))
            .collect()
    }

    pub fn to_bytes(v: &[Bit]) -> Vec<u8> {
        v
            .chunks(8)
            .map(|chunk| u8_from_bits(chunk))
            .collect()
    }


    pub fn bits_from_u8(num: u8) -> Vec<Bit> {
        let mut res = vec![];
        for i in 0 .. 8 {
            if num & (1 << i) == 1 << i {
                res.push(Bit::One);
            } else {
                res.push(Bit::Zero);
            }
        }

        res
    }

    pub fn u8_from_bits(bits: &[Bit]) -> u8 {
        let mut res = 0;
        for (i, bit) in bits.iter().enumerate() {
            if *bit == Bit::One {
                res |= 1 << i;
            }
        }

        res
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_from_bytes() {
            use Bit::*;
            assert_eq!(from_bytes(&[0]), vec![Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero]);
            assert_eq!(from_bytes(&[1, 2, 3]), vec![
                One, Zero, Zero, Zero, Zero, Zero, Zero, Zero,
                Zero, One, Zero, Zero, Zero, Zero, Zero, Zero,
                One, One, Zero, Zero, Zero, Zero, Zero, Zero,
            ]);
        }

        #[test]
        fn test_to_bytes() {
            use Bit::*;
            assert_eq!(to_bytes(&[
                One, Zero, Zero, Zero, Zero, Zero, Zero, Zero,
                Zero, One, Zero, Zero, Zero, Zero, Zero, Zero,
                One, One, Zero, Zero, Zero, Zero, Zero, Zero,
                ]),
                vec![1, 2, 3]);
        }

        #[test]
        fn test_u8_from_bits() {
            use Bit::*;
            assert_eq!(u8_from_bits(&[One]), 1);
            assert_eq!(u8_from_bits(&[One, One]), 3);
            assert_eq!(u8_from_bits(&[One, One, One]), 7);
            assert_eq!(u8_from_bits(&[Zero, One, One]), 6);
        }

        #[test]
        fn test_bits_from_u8() {
            use Bit::*;
            assert_eq!(bits_from_u8(1), vec![One, Zero, Zero, Zero, Zero, Zero, Zero, Zero]);
            assert_eq!(bits_from_u8(3), vec![One, One, Zero, Zero, Zero, Zero, Zero, Zero]);
            assert_eq!(bits_from_u8(7), vec![One, One, One, Zero, Zero, Zero, Zero, Zero]);
            assert_eq!(bits_from_u8(6), vec![Zero, One, One, Zero, Zero, Zero, Zero, Zero]);
        }
    }

}

mod parser {
    use super::*;

    pub fn parse(code: &str) -> Vec<Instruction> {
        code
            .chars()
            .filter_map(|ch| parse_instruction(ch))
            .collect()
    }

    fn parse_instruction(ch: char) -> Option<Instruction> {
        use Instruction::*;

        match ch {
            '+' => Some(Flip),
            ',' => Some(Read),
            ';' => Some(Write),
            '<' => Some(MoveLeft),
            '>' => Some(MoveRight),
            '[' => Some(SkipRight),
            ']' => Some(SkipLeft),
            _ => None
        }
    }
}

mod interpreter {
    use std::collections::HashMap;
    use super::*;

    pub struct Interpreter {
        tape: Vec<Bit>,
        output: Vec<Bit>,
        input: Vec<Bit>,
        program: Vec<Instruction>,
        program_pointer: usize,
        pointer: usize,
        matches: HashMap<usize, usize>
    }

    impl Interpreter {
        pub fn new(program: Vec<Instruction>, input: Vec<Bit>) -> Interpreter {
            Interpreter {
                tape: vec![Bit::Zero],
                pointer: 0,
                program_pointer: 0,
                output: vec![],
                matches: HashMap::new(),
                input,
                program,
            }
        }

        pub fn interpret(&mut self) {
            use Instruction::*;
            self.create_matches();

            while self.program_pointer < self.program.len() {
                match self.program[self.program_pointer] {
                    Flip => self.flip(),
                    Read => self.read(),
                    Write => self.write(),
                    MoveLeft => self.move_left(),
                    MoveRight => self.move_right(),
                    SkipLeft => self.skip_left(),
                    SkipRight => self.skip_right(),
                }
            }
        }

        pub fn get_output(&self) -> &Vec<Bit> {
            &self.output
        }

        fn get_tape(&self) -> &Vec<Bit> {
            &self.tape
        }

        fn create_matches(&mut self) {
            let mut matches = vec![];

            for (i, instr) in self.program.iter().enumerate() {
                match instr {
                    &Instruction::SkipRight => {
                        matches.push(i);
                    },
                    &Instruction::SkipLeft => {
                        let prev_i = matches.pop().unwrap();
                        self.matches.insert(prev_i, i);
                        self.matches.insert(i, prev_i);
                    },
                    _ => {}
                }
            }
        }

        fn get_matching_pointer(&self, i: usize) -> usize {
            *self.matches.get(&i).unwrap()
        }

        fn flip(&mut self) {
            self.tape[self.pointer] = self.tape[self.pointer].flip();
            self.program_pointer += 1;
        }

        fn read(&mut self) {
            self.tape[self.pointer] = if self.input.len() == 0 {
                Bit::Zero
            } else {
                self.input.remove(0)
            };

            self.program_pointer += 1;
        }

        fn write(&mut self) {
            self.output.push(self.tape[self.pointer]);
            self.program_pointer += 1;
        }

        fn move_left(&mut self) {
            if self.pointer == 0 {
                self.tape.insert(0, Bit::Zero);
            } else {
                self.pointer -= 1;
            }

            self.program_pointer += 1;
        }

        fn move_right(&mut self) {
            if self.pointer == self.tape.len() - 1 {
                self.tape.push(Bit::Zero);
            }

            self.pointer += 1;
            self.program_pointer += 1;
        }

        fn skip_left(&mut self) {
            if self.tape[self.pointer] == Bit::One {
                self.program_pointer = self.get_matching_pointer(self.program_pointer);
            } else {
                self.program_pointer += 1;
            }
        }

        fn skip_right(&mut self) {
            if self.tape[self.pointer] == Bit::Zero {
                self.program_pointer = self.get_matching_pointer(self.program_pointer);
            } else {
                self.program_pointer += 1;
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_flip() {
            use Instruction::*;
            use Bit::*;

            let mut interpreter = Interpreter::new(vec![Flip, Flip, Flip], vec![]);
            interpreter.interpret();
            assert_eq!(interpreter.get_tape(), &vec![One]);
        }

        #[test]
        fn test_move_left() {
            use Instruction::*;
            use Bit::*;

            let mut interpreter = Interpreter::new(vec![MoveLeft], vec![]);
            interpreter.interpret();
            assert_eq!(interpreter.get_tape(), &vec![Zero, Zero]);

            let mut interpreter = Interpreter::new(vec![MoveLeft, MoveLeft, MoveLeft], vec![]);
            interpreter.interpret();
            assert_eq!(interpreter.get_tape(), &vec![Zero, Zero, Zero, Zero]);
        }

        #[test]
        fn test_move_right() {
            use Instruction::*;
            use Bit::*;

            let mut interpreter = Interpreter::new(vec![MoveRight], vec![]);
            interpreter.interpret();
            assert_eq!(interpreter.get_tape(), &vec![Zero, Zero]);

            let mut interpreter = Interpreter::new(vec![MoveRight, MoveRight, MoveRight], vec![]);
            interpreter.interpret();
            assert_eq!(interpreter.get_tape(), &vec![Zero, Zero, Zero, Zero]);
        }

        #[test]
        fn test_skip() {
            use Instruction::*;
            use Bit::*;

            let mut interpreter = Interpreter::new(vec![SkipRight, Flip, SkipLeft], vec![]);
            interpreter.interpret();
            assert_eq!(interpreter.get_tape(), &vec![Zero]);

            let mut interpreter = Interpreter::new(
                vec![
                    Flip,
                    MoveRight,
                    MoveRight,
                    MoveRight,
                    Flip,
                    MoveLeft,
                    MoveLeft,
                    MoveLeft,
                    SkipRight,
                    MoveRight,
                    Flip,
                    SkipLeft
                ],
                vec![]
            );
            interpreter.interpret();
            assert_eq!(interpreter.get_tape(), &vec![One, One, One, Zero]);
        }

        #[test]
        fn test_read() {
            use Instruction::*;
            use Bit::*;

            let mut interpreter = Interpreter::new(vec![Read, MoveRight, Read, MoveRight, Read], vec![One, One, One]);
            interpreter.interpret();
            assert_eq!(interpreter.get_tape(), &vec![One, One, One]);
        }

        #[test]
        fn test_write() {
            use Instruction::*;
            use Bit::*;

            let mut interpreter = Interpreter::new(vec![Read, Write, MoveRight, Read, Write, MoveRight, Read, Write], vec![One, One, One]);
            interpreter.interpret();
            assert_eq!(interpreter.get_output(), &vec![One, One, One]);
        }
    }
}
