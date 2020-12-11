#!/usr/bin/env python

import colorsys, math, random
import pygame as pg

pg.init()


def get_time():
    """Get time in seconds."""
    return pg.time.get_ticks() // 1000


class BalanceAccount:
    def __init__(self, balance):
        self.balance = balance


class TaskAuction:
    def __init__(self, caller, deposit, description, pay_multiplier, jury,
                 duration, extension):
        caller.balance -= deposit
        self.balance = deposit
        self.description = description
        self.pay_multiplier = pay_multiplier
        self.current_bid = deposit // (pay_multiplier + 1)
        self.contractor = self
        self.client = caller
        self.jury = jury
        self.deadline = get_time() + duration
        self.extension = extension

    def extend(self, caller, extension):
        assert (caller == self.client)
        assert (self.accepting_bids() or self.contractor == self)
        self.deadline += extension
        return self.deadline

    def bid(self, caller, deposit):
        assert (self.accepting_bids())
        assert (deposit * 2 > self.current_bid)
        assert (deposit * 100 < self.current_bid * 99)
        assert (caller != self.jury)
        assert (caller != self.contractor)
        self.current_bid = deposit
        self.contractor = caller
        deadline = get_time() + self.extension
        if deadline > self.deadline:
            self.deadline = deadline

    def confirm(self, caller, value):
        assert (not self.accepting_bids())
        assert (caller == self.client or caller == self.contractor
                or caller == self.jury)
        payment = self.current_bid + self.get_current_pay()
        self.contractor.balance += payment
        self.client.balance += self.balance - payment

    def accepting_bids(self):
        return get_time() < self.deadline

    def get_contractor(self):
        return self.contractor

    def get_current_pay(self):
        return self.current_bid * self.pay_multiplier


class Agent:
    def __init__(self, size, speed, pos):
        self.account = BalanceAccount(0)
        self.size = size
        self.speed = speed
        self.pos = pos
        self.rect = pg.Rect(0, 0, 0, 0)
        self.color = tuple(
            round(i * 255)
            for i in colorsys.hsv_to_rgb(random.uniform(0, 1), 1, 0.9))
        self.load = 0

    def display(self, screen):
        new_rect = pg.draw.circle(screen, self.color, self.pos, self.size)
        dirty = new_rect.union(self.rect)
        self.rect = new_rect
        pg.draw.circle(screen, (180, ) * 3, self.pos,
                       self.size * math.sqrt(self.load / self.size**2))
        return dirty


class Item:
    font = pg.font.SysFont(None, 20)

    def __init__(self, price, size, pos):
        self.size = size
        self.pos = pos
        self.rect = pg.Rect(0, 0, 0, 0)
        self.color = (200, ) * 3
        self.price = price

    def display(self, screen):
        top_left = self.pos[0] - self.size, self.pos[1] - self.size
        new_rect = pg.draw.rect(
            screen, self.color,
            pg.Rect(top_left, (2 * self.size, 2 * self.size)))
        text = Item.font.render(str(self.price), True, (0, 0, 0))
        dirty = screen.blit(text, top_left).union(new_rect.union(self.rect))
        self.rect = new_rect
        return dirty


class Simulation:
    def __init__(self, screen_size, agents, item_limit, price_variance):
        # fields
        self.account = BalanceAccount(0)
        self.agents = agents
        self.max_agent_size = max(agents, key=lambda x: x.size).size
        self.min_agent_size = min(agents, key=lambda x: x.size).size
        self.item_limit = item_limit
        self.price_variance = price_variance
        self.items = {}
        self.auctions = {}
        self.entity_updates = []
        self.font = pg.font.SysFont(None, 24)
        self.counter = 0

        # create window
        self.screen = pg.display.set_mode(screen_size)
        pg.display.set_caption("VRP Sim")

        # create background
        self.background = pg.Surface(self.screen.get_size())
        self.background = self.background.convert()
        self.background.fill((255, ) * 3)
        pg.draw.circle(self.background, (0, ) * 3,
                       [coord / 2 for coord in self.screen.get_size()], 4)
        self.screen.blit(self.background, (0, 0))
        pg.display.flip()

    def update(self):
        for event in pg.event.get():
            if event.type == pg.QUIT or (event.type == pg.KEYDOWN
                                         and event.key == pg.K_ESCAPE):
                return False
        self.spawn_items()
        # display entities
        self.screen.blit(self.background, (0, 0))
        self.entity_updates += [
            entity.display(self.screen)
            for entity in list(self.items.values()) + self.agents
        ]
        # display scores
        for i, agent in enumerate(self.agents):
            score = self.font.render(
                f"(R {agent.size}, V {agent.speed}, P {agent.account.balance})",
                True, agent.color)
            self.entity_updates.append(self.screen.blit(score, (0, i * 20)))

        pg.display.update(self.entity_updates)
        self.entity_updates.clear()
        return True

    def run(self, hz):
        clock = pg.time.Clock()
        while self.update():
            clock.tick(hz)

    def spawn_items(self):
        while len(self.items) < self.item_limit:
            size = random.randrange(self.min_agent_size // 2,
                                    self.max_agent_size + 1)
            pos = (random.randrange(0,
                                    self.screen.get_size()[0]),
                   random.randrange(0,
                                    self.screen.get_size()[1]))
            price = int(
                math.sqrt((pos[0] - self.screen.get_size()[0] // 2)**2 +
                          (pos[1] - self.screen.get_size()[1] // 2)**2)
            ) * size * size * random.randrange(1, self.price_variance + 1)
            item_id = str(self.counter)
            self.items[item_id] = (Item(price, size, pos))
            self.auctions[item_id] = TaskAuction(self.account, 2 * price,
                                                 item_id, 1, self.account, 5,
                                                 1)
            self.counter += 1


def main():
    screen_size = (800, 800)
    agents = [
        Agent(random.randrange(10, 20), random.randrange(1, 5),
              (random.randrange(
                  0, screen_size[0]), random.randrange(0, screen_size[1])))
        for i in range(5)
    ]
    sim = Simulation(screen_size, agents, item_limit=30, price_variance=1)
    sim.run(10)


if __name__ == "__main__":
    main()
