#!/usr/bin/env python
import sys
import os
import re
import json
import random
import click

from server import Server


def solve(problem_name: str, mock: bool):
    if problem_name == "probatio":
        solve_probatio(mock=mock)
    else:
        raise ValueError(f"Unknown problem name: {problem_name}")
        # print(f'Problem "{problem_name}" is not supported')

DOORS = 6

def solve_probatio(mock: bool = False):
    server = Server(mock=mock)
    server.select(problem_name="probatio")

    N = server.N

    doors_from = [[[] for i in range(N)] for j in range(N)]  # [from][to] = [doors on from_side]

    all_doors = [str(ch) for ch in range(DOORS)]
    # plan = "".join([str(random.randint(0, DOORS-1)) for i in range(PLAN_LENGTH)])
    PLAN_LENGTH = 48  # 上限は18n

    tmp = all_doors * (PLAN_LENGTH // DOORS)

    random.shuffle(tmp)
    plan = "".join(tmp)
    print("plan:", plan)

    # find startingRoom
    response = server.explore([plan])
    results = response["results"]
    result = results[0]
    starting_room = result[0]

    checked = [[False for i in range(DOORS)] for j in range(N)]  # [room][door]
    rest = N * DOORS

    from_room = starting_room
    for i, (from_door_, to_room_) in enumerate(zip(plan, result[1:])):
        to_room = int(to_room_)
        from_door = int(from_door_)
        # print(" ", i, ")", from_room, from_door, "->", to_room)
        # visited[to_room] = True
        if not checked[from_room][from_door]:
            doors_from[from_room][to_room].append(from_door)
            checked[from_room][from_door] = True
            rest -= 1
            if rest == 0:
                print("Fulfilled. i =", i)
                break
        from_room = to_room

    if rest > 0:
        print("Not fulfilled with |plan| =", PLAN_LENGTH)
        sys.exit(0)

    connections = []
    for room in range(N):
        for door in doors_from[room][room]:
            connections.append((room, door, room, door))
            print(f"connect {room}.{door} -> self")

    for from_room in range(N):
        for to_room in range(from_room+1, N):
            from_side = doors_from[from_room][to_room]
            to_side = doors_from[to_room][from_room]
            assert len(from_side) == len(to_side)
            for from_door, to_door in zip(from_side, to_side):
                connections.append((from_room, from_door, to_room, to_door))
                print(f"connect {from_room}.{from_door} <-> {to_room}.{to_door}")

    verdict = server.guess(starting_room, connections)
    print("VERDICT:", verdict)


@click.command()
@click.option("-m", "--mock", is_flag=True, default=False)
def main(mock: bool):
    solve_probatio(mock=mock)


if __name__ == '__main__':
    main()
