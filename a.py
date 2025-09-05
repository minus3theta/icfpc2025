#!/usr/bin/env python
import sys
import os
import re
import json
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

    # explore
    N = server.N
    visited = [False for i in range(N)]
    explored_from = [False for i in range(N)]
    route_to = [None for i in range(N)]
    result_from_room = [None for i in range(N)]
    # passed_count = [[0 for i in range(N)] for j in range(N)]
    doors_from = [[[] for i in range(N)] for j in range(N)]  # [from][to] = [doors on from_side]

    visited[0] = True
    route_to[0] = ""

    def explore_from_here(here):
        plans = [route_to[here] + str(i) for i in range(DOORS)]  # from 0
        response = server.explore(plans)
        results = response["results"]
        query_count = response["queryCount"]
        # print("exploring from", here, ":", results)
        print("queryCount:", query_count)
        result_from_room[here] = [result[-1] for result in results]
        print("result_from_room", here, ":", result_from_room[here])
        assert len(result_from_room[here]) == DOORS
        return result_from_room[here]

    for depth in range(N):
        for here in range(N):
            if visited[here] and not explored_from[here]:
                print(f"d={depth}, exploring from {here} ;", route_to[here])
                assert route_to[here] is not None
                results = explore_from_here(here)
                for door_no in range(DOORS):
                    to_room = results[door_no]
                    visited[to_room] = True
                    doors_from[here][to_room].append(door_no)
                    if route_to[to_room] is None:
                        route_to[to_room] = route_to[here] + str(door_no)
                explored_from[here] = True

    if not visited[1] and not visited[2]:
        raise ValueError(f"Unsolvable because all the doors of Room#0 leads to itself")

    if not visited[1]:
        doors_from[1][1] = range(DOORS)
    if not visited[2]:
        doors_from[2][2] = range(DOORS)

    for from_room in range(N):
        for to_room in range(N):
            print(f"doors_from[{from_room}][{to_room}]:", doors_from[from_room][to_room])

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

    # print("connection:", connections)

    verdict = server.guess(connections)
    print("VERDICT:", verdict)


@click.command()
def main():
    # solve("probatio")
    solve_probatio()


if __name__ == '__main__':
    main()
