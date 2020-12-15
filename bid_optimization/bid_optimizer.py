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
        return auction.get_current_pay() - self.cost_function(
            auction, self.participating_auctions)

    def check_auction_results(self):
        won_auctions = []
        for auction_id, auction in list(self.on_going_auctions.items()):
            # delete expired auctions
            if not auction.accepting_bids():
                # check if you won the auction
                if auction.get_contractor() == self.caller:
                    won_auctions.append(auction)
                del self.on_going_auctions[auction_id]
                del self.utilities[auction_id]
                continue
        return won_auctions

    def evaluate_and_bid(self, recompute_utilities=False):
        best_auctions = [None, None]
        for auction_id, auction in self.on_going_auctions.items():
            if recompute_utilities:
                self.utilities[auction_id] = self.utility_function(auction)
            # reject negative utilities
            if self.utilities[auction_id] <= 0:
                continue
            # ignore already participating auctions
            if auction_id in self.participating_auctions:
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
            # initially set price equivalent to break even cost
            bid_price = best_auction.get_current_pay() - self.utilities[
                best_auctions[0]]
            # increase price such that utility is equivalent to second best option
            if best_auctions[1] is not None:
                bid_price += self.utilities[best_auctions[1]]
            # scale bid to expected deposit range
            bid_price //= best_auction.pay_multiplier
            bid_price = min(bid_price,
                            (best_auction.get_current_bid() * 99 // 100) - 1)
            bid_price = max(bid_price,
                            (best_auction.get_current_bid() // 2) + 1)
            # TODO handle bidding errors
            try:
                best_auction.bid(self.caller, bid_price)
            except AssertionError:
                return None
            self.participating_auctions[best_auctions[0]] = best_auction
        return best_auctions[0]

    def on_auction_update(self, auction_id, auction):
        '''possible events: creation, bid, cancel,    extend, confirm'''
        # check if participating auction was out bid
        if auction_id in self.participating_auctions and auction.get_contractor(
        ) != self.caller:
            del self.participating_auctions[auction_id]
        # store updated auction
        self.on_going_auctions[auction_id] = auction
        self.utilities[auction_id] = self.utility_function(auction)
