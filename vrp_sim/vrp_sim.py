#!/usr/bin/env python3

import colorsys, math, random
import pygame as pg
import bid_optimizer

pg.init()


def get_time():
    """Get time in seconds."""
    return pg.time.get_ticks() // 1000


def distance(a, b):
    return math.sqrt((a[0] - b[0])**2 + (a[1] - b[1])**2)


def inc_towards(a, b, inc):
    dx = b[0] - a[0]
    dy = b[1] - a[1]
    dz = math.sqrt(dx**2 + dy**2)
    dx *= inc / dz
    dy *= inc / dz
    return (a[0] + dx, a[1] + dy)


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

    def extend(self, caller, deposit, extension):
        assert (caller == self.client)
        self.balance += deposit
        if self.contractor == self:
            self.current_bid = self.balance // (self.pay_multiplier + 1)
        else:
            assert (self.accepting_bids())
        self.deadline += extension
        return self.deadline

    def bid(self, caller, deposit):
        assert (self.accepting_bids())
        assert (deposit * 2 >= self.current_bid)
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
        self.contractor.balance += self.get_current_pay()
        self.client.balance += self.balance - self.get_current_pay()
        self.balance = 0

    def accepting_bids(self):
        return get_time() < self.deadline

    def get_contractor(self):
        return self.contractor

    def get_current_pay(self):
        return self.current_bid * self.pay_multiplier

    def get_current_bid(self):
        return self.current_bid * self.pay_multiplier


class Agent:
    def __init__(self, size, speed, pos, color, dropoff):
        self.account = BalanceAccount(0)
        self.size = size
        self.speed = speed
        self.pos = pos
        self.rect = pg.Rect(0, 0, 0, 0)
        self.color = color
        self.itinerary = []
        self.fill = 0
        self.load = 0
        self.next_itinerary = []
        self.won_auctions = []
        self.bid_optimizer = bid_optimizer.BidOptimizer(
            self.account, self.cost_function)
        self.items = {}
        self.dropoff = dropoff
        self.total = 0

    def find_best_slot(self, auction):
        best_slot = (None, float('inf'))
        for i in range(len(self.next_itinerary) + 1):
            pos = self.items[auction.description].pos
            p1 = self.dropoff if i == 0 else self.items[self.next_itinerary[
                i - 1].description].pos
            p2 = self.dropoff if i == len(self.next_itinerary) else self.items[
                self.next_itinerary[i].description].pos
            d = distance(pos, p1) + distance(pos, p2) - distance(p1, p2)
            if d < best_slot[1]:
                best_slot = (i, d)
        return best_slot

    def load_exceed(self, item):
        return self.load + item.size**2 > self.size**2

    def cost_function(self, auction, participating_auctions):
        item = self.items.get(auction.description)
        if item is None or self.load_exceed(item):
            return float('inf')
        i, d = self.find_best_slot(auction)
        return round(d / self.speed)

    def update(self):
        # update won and outbid auctions
        self.won_auctions += self.bid_optimizer.check_auction_results()
        # update itineray if outbid
        for auction in list(self.next_itinerary):
            if auction.description not in self.bid_optimizer.participating_auctions:  # and auction not in self.won_auctions:
                self.load -= self.items[auction.description].size**2
                self.next_itinerary.remove(auction)
        # pick up items once all actions are confirmed
        if not self.itinerary and len(self.won_auctions) == len(
                self.next_itinerary):
            for auction in self.next_itinerary:
                item = self.items[auction.description]
                item.color = tuple(x * 0.5 for x in item.color)
            self.itinerary = list(self.next_itinerary + [None])
            self.itinerary.reverse()
            self.won_auctions = []
            self.next_itinerary = []
            self.load = 0
        # move towards item to pickup
        if self.itinerary:
            if self.itinerary[-1] is None:
                item = Item(0, 0, self.dropoff)
            else:
                item_id = self.itinerary[-1].description
                item = self.items[item_id]
            self.pos = inc_towards(self.pos, item.pos, self.speed)
            # picked up item
            if distance(self.pos, item.pos) < self.size + item.size:
                if self.itinerary[-1]:
                    self.fill += item.size**2
                    self.total += item.size**2
                    self.itinerary[-1].confirm(self.account, True)
                    del self.items[item_id]
                else:
                    self.fill = 0
                del self.itinerary[-1]
        else:
            self.pos = inc_towards(self.pos, self.dropoff, self.speed)

        bid_auction = self.bid_optimizer.evaluate_and_bid(True)
        if bid_auction is not None:
            auction = self.bid_optimizer.participating_auctions[bid_auction]
            item = self.items[auction.description]
            item.color = self.color
            item.price = round(auction.get_current_pay())
            i, d = self.find_best_slot(auction)
            self.next_itinerary.insert(i, auction)
            self.load += item.size**2
            #print(d, (self.size**2), self.load,
            #      [auction.description for auction in self.next_itinerary])
        return bid_auction

    def display(self, screen):
        new_rect = pg.draw.circle(screen, self.color, self.pos, self.size)
        dirty = new_rect.union(self.rect)
        self.rect = new_rect
        pg.draw.circle(screen, (180, ) * 3, self.pos,
                       self.size * math.sqrt(self.fill / self.size**2))
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
    def __init__(self, screen_size, agents, item_limit, price_variance,
                 duration):
        # fields
        self.account = BalanceAccount(0)
        self.item_limit = item_limit
        self.price_variance = price_variance
        self.items = {}
        self.auctions = {}
        self.agents = agents
        self.max_agent_size = max(agents, key=lambda x: x.size).size
        self.min_agent_size = min(agents, key=lambda x: x.size).size
        self.entity_updates = []
        self.font = pg.font.SysFont(None, 24)
        self.counter = 0
        self.duration = duration
        self.pause = False

        for agent in self.agents:
            agent.items = self.items

        # create window
        self.screen = pg.display.set_mode(screen_size)
        pg.display.set_caption("VRP Sim")

        # create background
        self.background = pg.Surface(self.screen.get_size())
        self.background = self.background.convert()
        self.background.fill((255, ) * 3)
        pg.draw.circle(self.background, (0, ) * 3,
                       [coord / 2 for coord in self.screen.get_size()], 5)
        self.screen.blit(self.background, (0, 0))
        pg.display.flip()

    def update(self):
        for event in pg.event.get():
            if event.type == pg.QUIT or (event.type == pg.KEYDOWN
                                         and event.key == pg.K_ESCAPE):
                return False
            elif event.type == pg.KEYDOWN and event.key == pg.K_SPACE:
                self.pause = not self.pause
                print("PAUSE", self.pause)
        if self.pause:
            return True
        self.spawn_items()
        self.update_agents()
        self.bump_items()
        # display entities
        self.screen.blit(self.background, (0, 0))
        self.entity_updates += [
            entity.display(self.screen)
            for entity in list(self.items.values()) + self.agents
        ]
        # display scores
        for i, agent in enumerate(self.agents):
            score = self.font.render(
                f"(R {agent.size}, V {agent.speed}, T {agent.total}, P {round(agent.account.balance)})",
                True, agent.color)
            self.entity_updates.append(self.screen.blit(score, (0, i * 20)))

        #pg.display.update(self.entity_updates)
        pg.display.flip()
        self.entity_updates.clear()
        return True

    def run(self, hz):
        clock = pg.time.Clock()
        while self.update():
            clock.tick(hz)

    def update_agents(self):
        for agent in self.agents:
            item_update = agent.update()
            if item_update is not None:
                auction_update = agent.bid_optimizer.participating_auctions[
                    item_update]
                self.auctions[item_update] = auction_update
                for other_agent in self.agents:
                    other_agent.bid_optimizer.on_auction_update(
                        item_update, auction_update)

    def bump_items(self):
        # extend expired auctions
        for item_id, auction in self.auctions.items():
            if not auction.accepting_bids() and auction.get_contractor(
            ) == auction:
                self.items[item_id].price *= 2
                auction.extend(self.account, self.items[item_id].price,
                               self.duration)
                for agent in self.agents:
                    agent.bid_optimizer.on_auction_update(item_id, auction)

    def spawn_items(self):
        while len(self.items) < self.item_limit:
            size = random.randrange(self.min_agent_size // 2,
                                    self.max_agent_size // 2 + 1)
            pos = (random.randrange(0,
                                    self.screen.get_size()[0]),
                   random.randrange(0,
                                    self.screen.get_size()[1]))
            price = int(4 * size**2 *
                        random.randrange(1, self.price_variance + 1))
            item_id = str(self.counter)
            self.items[item_id] = (Item(price, size, pos))
            self.auctions[item_id] = TaskAuction(self.account, 2 * price,
                                                 item_id, 1, self.account,
                                                 self.duration, 1)
            # notify agents of new auction
            for agent in self.agents:
                agent.bid_optimizer.on_auction_update(item_id,
                                                      self.auctions[item_id])
            self.counter += 1


def main():
    screen_size = (800, 800)
    agents = [
        Agent(size=random.randrange(15, 30),
              speed=random.randrange(4, 8),
              pos=(random.randrange(0, screen_size[0]),
                   random.randrange(0, screen_size[1])),
              color=tuple(
                  round(val * 255)
                  for val in colorsys.hsv_to_rgb(i / 6, 1, 0.9)),
              dropoff=tuple(x / 2 for x in screen_size)) for i in range(6)
    ]
    sim = Simulation(screen_size,
                     agents,
                     item_limit=30,
                     price_variance=1,
                     duration=2)
    sim.run(30)


if __name__ == "__main__":
    main()
