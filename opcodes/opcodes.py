import json

operands_map = {
    "A": "(Register::A)",
    "B": "(Register::B)",
    "C": "(Register::C)",
    "D": "(Register::D)",
    "E": "(Register::E)",
    "F": "(Register::F)",
    "H": "(Register::H)",
    "L": "(Register::L)",
    "SP": "(Register::SP)",
    "a16": "U16",
    "a8": "U8",
    "n16": "U16",
    "n8": "U8",
    "e8": "I8"
}

flags_map = {
    "NZ": "(Flag::NZ)",
    "Z": "(Flag::Z)",
    "NC": "(Flag::NC)",
    "C": "(Flag::C)",
    "H": "(Flag::H)",
    "N": "(Flag::N)"
}

mnemonics_map = {
    "a16": "u16",
    "a8": "u8",
    "n16": "u16",
    "n8": "u8",
    "e8": "i8"
}

opcodes_with_flag = [
    "JR", 
    "JP", 
    "RET", 
    "CALL"
]

def split_union_register(st):
    reg1 = st[0]
    reg2 = st[1]

    return f"({operands_map[reg1]},{operands_map[reg2]})"

def get_operand(op):
    is_address = not op["immediate"]

    if op["name"] in operands_map.keys():
        return f"[{operands_map[op["name"]]}]" if is_address else operands_map[op["name"]]
    
    if len(op["name"]) == 2 and op["name"][0] in operands_map.keys() and op["name"][1] in operands_map.keys():
        return f"[{split_union_register(op["name"])}]" if is_address else f"{split_union_register(op["name"])}"
    else: 
        op_name: str = op["name"]
        if op_name[0] == "$":
            op_name = op_name.replace("$", "0x")
        return f"[(Literal({op_name}))]" if is_address else f"(Literal({op_name}))"

def get_operands(ops, is_flag):
    operands = []
    
    if is_flag:
        operands.append(flags_map[ops[0]["name"]])
        ops = ops[1:]
    
    for op in ops:
        operands.append(get_operand(op))
    return ",".join(operands)


opcodes = json.load(open("data/opcodes.json"))
unprefixed = opcodes["unprefixed"]
prefixed = opcodes["cbprefixed"]

'''
NOTE: This program does not handle the construction of the opcode 
LD HL, SP + e8 and constructs it as LD HL, SP, e8, so you have to 
correct it manually.
'''
def get_translation(map, is_optional):
    res = []

    for key in map.keys():
        operands = ""

        if is_optional and "ILLEGAL" == map[key]["mnemonic"][:-3]:
            res.append("None")
            continue
        else:
            operands = get_operands(
                map[key]["operands"], 
                (map[key]["mnemonic"] in opcodes_with_flag and len(map[key]["operands"]) == 2) or 
                (map[key]["mnemonic"] == "RET" and len(map[key]["operands"]) == 1)
            )

        operands_mnemonic = []

        for op in map[key]["operands"]:
            name = op["name"]
            
            if name in mnemonics_map.keys():
                name = mnemonics_map[name]

            if "decrement" in op.keys():
                name += "-"
            elif "increment" in op.keys():
                name += "+"
            
            if not op["immediate"]:
                name = f"[{name}]"

            operands_mnemonic.append(name)

        mnemonic = map[key]["mnemonic"] + " " + ", ".join(operands_mnemonic)
        res.append(
            f"Some(Opcode::new({key}, \"{mnemonic}\", operand!({operands}), vec!{map[key]["cycles"]}))" if is_optional
            else f"Opcode::new({key}, \"{mnemonic}\", operand!({operands}), vec!{map[key]["cycles"]})"   
        )

    return res

unprefixed_translation = get_translation(unprefixed, True)

prefixed_translation = get_translation(prefixed, False)



with open("data/opcodes_translations.txt", "w") as file:
    unprefixed_st = ",\n".join(unprefixed_translation) + "\n"*3
    if file.write(unprefixed_st) != len(unprefixed_st):
        raise RuntimeError()
    prefixed_st = ",\n".join(prefixed_translation)
    if file.write(prefixed_st) != len(prefixed_st):
        raise RuntimeError()

    