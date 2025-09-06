#!/usr/bin/env python
import sys
import os
import re
import json
import click

from server import Server


def solve(problem_name):
    if problem_name == "probatio":
        solve_probatio()
    else:
        raise ValueError(f"Unknown problem name: {problem_name}")


def solve_probatio(mock: bool = False):
    server = Server(mock=mock)
    server.select(problem_name="probatio")

    # explore
    N = server.N
    DOORS = 6

    visited = [False for i in range(N)]
    explored_from = [False for i in range(N)]
    route_to = [None for i in range(N)]
    result_from_room = [None for i in range(N)]
    doors_from = [[[] for i in range(N)] for j in range(N)]  # [from][to] = [doors on from_side]

    response = server.explore(["0"])
    starting_room = response["results"][0][0]
    print("starting_room:", response, starting_room)

    visited[starting_room] = True
    route_to[starting_room] = ""

    def explore_from_here(here):
        plans = [route_to[here] + str(i) for i in range(DOORS)]  # from 0
        response = server.explore(plans)
        results = response["results"]
        query_count = response["queryCount"]
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

    # check unvisited rooms
    unvisited_count = 0
    for room in range(3):
        if not visited[room]:
            unvisited_count += 1
            doors_from[room][room] = range(DOORS)

    assert 0 <= unvisited_count <= 2
    if unvisited_count == 2:
        raise ValueError(f"Unsolvable because all the doors of Room#0 leads to itself")

    for from_room in range(N):
        for to_room in range(N):
            print(f"doors_from[{from_room}][{to_room}]:", doors_from[from_room][to_room])

    # make graph
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

    # submit our guess
    verdict = server.guess(starting_room, connections)
    print("VERDICT:", verdict)


@click.command()
@click.option("-m", "--mock", is_flag=True, default=False)
def main(mock: bool):
    solve_probatio(mock=mock)


if __name__ == '__main__':
    main()
