#!/usr/bin/env python
import sys
import os
import re
import json
import random
import requests
import subprocess
import tempfile
import click
from typing import List


class Server:
    def __init__(self):
        self.url = "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com"
        self.our_id = ""

    def select(self, problem_name: str):
        if problem_name == "probatio":
            self.N = 3
        else:
            raise ValueError(f"Unknown problem name: {problem_name}")

        payload = {
            "id": self.our_id,
            "problemName": problem_name,
        }
        response = requests.post(self.url + "/select", json=payload)
        data = response.json()
        assert data["problemName"] == problem_name
        return data

    def explore(self, plans: List[str]):
        payload = {
            "id": self.our_id,
            "plans": plans,
        }
        response = requests.post(self.url + "/explore", json=payload)
        data = response.json()
        # print(data)
        assert "results" in data
        assert "queryCount" in data
        return data

    def guess(self, connections):
        # [[0 1 2 1], ...]
        rooms = list(range(self.N))
        starting_room = 0

        connections_map = []
        for (from_room, from_door, to_room, to_door) in connections:
            connections_map.append(
                {
                    "from": {
                        "room": from_room,
                        "door": from_door,
                    },
                    "to": {
                        "room": to_room,
                        "door": to_door,
                    },
                }
            )

        payload = {
            "id": self.our_id,
            "map": {
                "rooms": rooms,
                "startingRoom": starting_room,
                "connections": connections_map,
            },
        }
        response = requests.post(self.url + "/guess", json=payload)
        data = response.json()
        assert "correct" in data
        return data["correct"]


def solve(problem_name):
    if problem_name == "probatio":
        solve_probatio()
    else:
        raise ValueError(f"Unknown problem name: {problem_name}")
        # print(f'Problem "{problem_name}" is not supported')

DOORS = 6

def solve_probatio():
    server = Server()
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

    response = server.explore([plan])
    results = response["results"]
    query_count = response["queryCount"]

    result = results[0]
    starting_room = result[0]

    checked = [[False for i in range(DOORS)] for j in range(N)]  # [room][door]
    rest = N * DOORS

    from_room = starting_room
    for i, (to_room_, from_door_) in enumerate(zip(result[1:], plan)):
        to_room = int(to_room_)
        from_door = int(from_door_)
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

    verdict = server.guess(connections)
    print("VERDICT:", verdict)


@click.command()
def main():
    solve_probatio()


if __name__ == '__main__':
    main()
