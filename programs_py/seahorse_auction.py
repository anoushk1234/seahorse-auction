# seahorse_auction
# Built with Seahorse v0.2.5

from seahorse.prelude import *



declare_id('Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS')

class Auction(Account):
    ongoing: bool
    seller: Pubkey
    item_holder: Pubkey
    currency_holder: Pubkey
    bidder: Pubkey
    price: u64

@instruction
def create_auction(auction: Empty[Auction], start_price: u64, payer: Signer,seller: UncheckedAccount, currency_holder: TokenAccount, item_holder: TokenAccount):
  
    auction = auction.init(payer,['auction', seller.key()])
    auction.ongoing = True
    auction.seller = seller.key()
    auction.item_holder = item_holder.key()
    auction.currency_holder = currency_holder.key()
    auction.bidder = seller.key()
    auction.price = start_price

@instruction
def bid(auction: Auction, price: u64, bidder: TokenAccount, authority: Signer, currency_holder: TokenAccount):
    assert(price <= auction.price), "Bid Price Too Low"
    assert(auction.currency_holder == currency_holder.key()), "Unauthorized Currency Holder"
    bidder.transfer(
        authority = authority,
        to= currency_holder,
        amount= price
    )
    
    auction.price = price
    auction.bidder = bidder.authority()

@instruction
def close_auction(auction: Auction, item_receiver: TokenAccount, item_holder: TokenAccount, currency_holder: TokenAccount,item_holder_auth: Signer, currency_holder_auth: Signer, currency_receiver: TokenAccount):
    assert(item_holder.key() == auction.item_holder), "Unauthorized Item Holder"
    assert(currency_holder.key() == auction.currency_holder), "Unauthorized Item Holder"
    assert(auction.bidder == item_receiver.authority()), "Receiver not Owner"

    item_holder.transfer(
        authority= item_holder_auth,
        to= item_receiver,
        amount= item_holder.amount()
    )
    if currency_holder.amount() >= auction.price:
        currency_holder.transfer(
            authority= currency_holder_auth,
            to= currency_receiver,
            amount= auction.price
        )
    auction.ongoing = False
    






