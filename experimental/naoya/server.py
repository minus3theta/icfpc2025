import requests
from typing import List, Tuple


class Server:
    def __init__(self, mock: bool = False):
        if mock:
            self.url = "http://localhost:8080"
            self.our_id = "Gon The Fox"
        else:
            self.url = "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com"
            self.our_id = "yone.j.synthesis@gmail.com nsRDwhEnX4yrlBibSH7pvw"

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
        print(data)
        assert data["problemName"] == problem_name
        return data

    def explore(self, plans: List[str]):
        payload = {
            "id": self.our_id,
            "plans": plans,
        }
        response = requests.post(self.url + "/explore", json=payload)
        data = response.json()
        print(data)
        assert "results" in data
        assert "queryCount" in data
        return data

    def guess(self, starting_room: int, connections: List[Tuple[int]]):
        # connections: [ (0 1 2 1), ... ]
        rooms = list(range(self.N))

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
        print(data)
        assert "correct" in data
        return data["correct"]
