use crate::eval::Object;
use crate::compiler::ByteCode;
use crate::code::convert_two_u8s_be_to_usize;

const STACK_SIZE : usize = 2048;

struct VM {
    instructions: Vec<u8>,
    constants: Vec<Object>,
    stack: [Object; STACK_SIZE],
    sp: usize, // stores the next FREE space on the stack
}

impl VM {
    fn new(byte_code: ByteCode) -> Self {
        VM {
            instructions: byte_code.instructions,
            constants: byte_code.constants,
            // we rely on the stack pointer to ensure we don't read uninitialized memory
            // this should have the same result as [Object::Null, STACK_SIZE] which is not allow because Object is not copy
            stack: unsafe { std::mem::zeroed() },
            sp: 0
        }
    }

    fn stack_peek(&self) -> &Object {
        // ignoring the potential of an empty stack
        &self.stack[self.sp - 1]
    }

    fn run(&mut self) {
        let mut ip = 0; // instruction pointer

        while ip < self.instructions.len() {
            let instruction_address = ip;
            ip += 1;

            match self.instructions[instruction_address] {
                0x01 => {
                    // OpConstant
                    let const_index = convert_two_u8s_be_to_usize(self.instructions[ip], self.instructions[ip + 1]);
                    ip += 2;
                    self.push(self.constants[const_index].clone());
                },
                0x02 => {
                    // OpAdd
                    match (self.pop(), self.pop()) {
                        (Object::Integer(right), Object::Integer(left)) => self.push(Object::Integer(left + right)),
                        _ => panic!("unhandled argument types to OpAdd"),
                    }
                }
                _ => panic!("unhandled instruction"),
            }
        }
    }

    fn push(&mut self, obj: Object) {
        self.stack[self.sp] = obj;
        self.sp += 1; // ignoring the potential stack overflow here
    }

    fn pop(&mut self) -> Object {
        // ignoring the potential of stack underflow here
        let obj = unsafe { std::mem::replace(&mut self.stack[self.sp - 1], std::mem::zeroed()) };
        self.sp -= 1;

        obj
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile_from_source;

    #[test]
    fn run_infix_add() {
        let input = "1 + 2;";
        let byte_code = compile_from_source(input);

        let mut vm = VM::new(byte_code);
        vm.run();

        assert_eq!(&Object::Integer(3), vm.stack_peek());
    }
}
