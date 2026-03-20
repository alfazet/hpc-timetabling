import os
import re

for file in os.listdir("../itc2019"):
    with open(f"../itc2019/{file}", "r") as f:
        lines = f.readlines()
    for line in lines:
        line = line.strip()
        tokens = line.split(" ")
        if tokens[0] == "<class" and "parent" in line:
            line = line[7:-1]
            line = re.split("=| ", line)
            # print(line)
            id_idx = line.index("id") + 1
            parent_idx = line.index("parent") + 1
            id = int(line[id_idx][1:-1])
            parent = int(line[parent_idx][1:-1])
            if id < parent:
                print(f"ID: {id}, Parent: {parent}")