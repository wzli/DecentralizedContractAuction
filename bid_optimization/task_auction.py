#!/usr/bin/env python3


class TaskAuction:
    def __init__(self, caller, deposit, description, pay_multiplier, jury,
                 duration, extension):
        pass

    def extend(self, caller, extension):
        pass

    def bid(self, caller, deposit):
        pass

    def cancel(self, caller):
        pass

    def confirm(self, caller, value):
        pass

    # predicates

    def accepting_bids(self):
        pass

    def in_dispute(self):
        pass

    # getters
    def get_description(self):
        pass

    def get_pay_multiplier(self):
        pass

    def get_current_bid(self):
        pass

    def get_current_pay(self):
        pass

    def get_contractor(self):
        pass

    def get_client(self):
        pass

    def get_jury(self):
        pass

    def get_deadline(self):
        pass

    def get_extension(self):
        pass

    def get_contractor_confirm(self):
        pass

    def get_client_confirm(self):
        pass
