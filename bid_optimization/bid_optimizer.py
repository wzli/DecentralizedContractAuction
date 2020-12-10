#!/usr/bin/env python3


class BidOptimizer:
    def __init__(self, caller, cost_function):
        self.caller = caller
        self.cost_function = cost_function
        self.participating_auctions = {}
        self.on_going_auctions = {}
        self.utilities = {}
        pass

    def utility_function(self, auction):
        return auction.get_current_pay() - self.cost_function(auction)

    def evaluate_and_bid(self, recompute_utilities=False):
        best_auctions = [None, None]
        won_auctions = []
        for auction_id, auction in on_going_auctions.items():
            # delete expired auctions
            if not auction.accepting_bids():
                if auction.get_contractor() == self.caller:
                    won_auctions.append(auction)
                del self.on_going_auctions[auction_id]
                del self.utilities[auction_id]
                continue
            if recompute_utilities:
                self.utilities[auction_id] = self.utility_function(
                    auction, self.participating_auctions)
            # reject negative utilities
            if self.utilities[auction_id] <= 0:
                continue
            # find the highest utility auctions
            if best_auctions[0] is None or self.utilities[
                    auction_id] > self.utilities[best_auctions[0]]:
                best_auctions[0] = auction_id
            elif best_auctions[1] is None or self.utilities[
                    auction_id] > self.utilities[best_auctions[1]]:
                best_auctions[1] = auction_id
        # bid on the best auction at a price of decision boundary between second best
        if best_auctions[0] is not None:
            best_auction = self.on_going_auctions[best_auctions[0]]
            # TODO handle bidding errors
            best_auction.bid(
                self.caller,
                best_auction.get_current_pay() -
                self.utilities[best_auctions[0]] +
                self.utilities[best_auctions[1]])
            self.participating_auctions[best_auctions[0]] = best_auction
        return won_auctions

    def on_auction_update(self, auction_id, auction):
        '''possible events: creation, bid, cancel,    extend, confirm'''
        # check if participating auction was out bid
        if auction_id in self.participating_auctions and auction.get_contractor(
        ) != self.caller:
            del self.participating_auctions[auction_id]
        # update utility only if bid was updated
        if auction_id not in self.on_going_auctions or auction.get_current_bid(
        ) != self.on_going_auctions[auction_id].get_current_bid():
            self.utilities[auction_id] = self.utility_function(
                auction, self.participating_auctions)
        # store updated auction
        self.on_going_auctions[auction_id] = auction
