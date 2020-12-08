#!/usr/bin/env python3


class BidOptimizer:
    def __init__(self, caller, cost_function):
        self.caller = caller
        self.cost_function = cost_function
        self.won_auctions = {}
        self.on_going_auctions = {}
        self.utilities = {}
        pass

    def utility_function(self, auction):
        return auction.get_current_pay() - self.cost_function(auction)

    def evaluate_and_bid(self, recompute_utilities=False):
        best_auctions = [None, None]
        for auction_id, auction in on_going_auctions.items():
            # delete expired auctions
            if not auction.accepting_bids():
                del self.on_going_auctions[auction_id]
                del self.utilities[auction_id]
                continue
            if recompute_utilities:
                self.utilities[auction_id] = self.utility_function(auction)
            # reject negative utilities
            if self.utilities[auction_id] <= 0:
                continue
            # find the highest utility auctions
            if best_auction[0] is None or self.utilities[
                    auction_id] > self.utilities[best_auction[0]]:
                best_auction[0] = auction_id
            elif best_auction[1] is None or self.utilities[
                    auction_id] > self.utilities[best_auction[1]]:
                best_auction[1] = auction_id
            # bid on the best auction at a price of decision boundary between second best
            if best_auction[0] is not None:
                # TODO handle bidding errors
                self.on_going_auctions[best_auction[0]].bid(
                    self.caller, self.utilities[best_auction[0]] -
                    self.utilities[best_auction[1]] -
                    self.on_going_auctions[best_auction[0]].get_current_pay)
                self.won_auctions[best_auction[0]] = self.on_going_auctions[
                    best_auction[0]]
                del self.on_going_auctions[best_auction[0]]
                return self.won_auctions[best_auction[0]]

    def on_auction_update(self, auction_id, auction):
        ''' creation, bid, extend, cancel, confirm'''
        if not auction.accepting_bids():
            return
        # update utility cache only if bid was updated
        if auction_id not in self.on_going_auctions or auction.get_current_bid(
        ) != self.on_going_auctions[auction_id].get_current_bid():
            self.utilities[auction_id] = self.utility_function(auction)

        self.on_going_auctions[auction_id] = auction
