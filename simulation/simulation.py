#!/usr/bin/env python

import random
import pygame as pg

class Agent:
    def __init__(self, size, pos = (0,0)):
        self.size = size
        self.pos = pos
        self.rect = pg.Rect(0,0,0,0)
        self.color = random.sample(range(20, 240), 3)
        self.load = 0

    def display(self, screen):
        new_rect = pg.draw.circle(screen, self.color, self.pos, self.size * 6 // 5)
        dirty = new_rect.union(self.rect)
        self.rect = new_rect
        pg.draw.circle(screen, (240,) * 3, self.pos, self.size - self.load)
        return dirty

class Item:
    def __init__(self, size, pos, price):
        self.size = size
        self.pos = pos
        self.rect = pg.Rect(0,0,0,0)
        self.color = (10,) * 3
        self.price = price

    def display(self, screen):
        new_rect = pg.draw.rect(screen, self.color, pg.Rect((self.pos[0] - self.size, self.pos[1] - self.size), (2 * self.size, 2 * self.size)))
        dirty = new_rect.union(self.rect)
        self.rect = new_rect
        return dirty

def main():
    # create window
    pg.init()
    screen = pg.display.set_mode((500, 500))
    pg.display.set_caption("Task Auction Simulation")

    # create background
    background = pg.Surface(screen.get_size())
    background = background.convert()
    background.fill((255,) * 3)
    pg.draw.circle(background, (0,) * 3, [coord/2 for coord in screen.get_size()], 3)
    screen.blit(background, (0, 0))
    pg.display.flip()

    # create agents
    agents = [Agent(10, (100,100))]
    items = [Item(10, (200,200), 100)]

    # main loop
    clock = pg.time.Clock()
    going = True
    while going:
        clock.tick(10)

        for event in pg.event.get():
            if event.type == pg.QUIT:
                going = False
            elif event.type == pg.KEYDOWN and event.key == pg.K_ESCAPE:
                going = False

        # display agents
        screen.blit(background, (0, 0))
        entity_updates = [entity.display(screen) for entity in agents + items]
        pg.display.update(entity_updates)

    pg.quit()

if __name__ == "__main__":
    main()
