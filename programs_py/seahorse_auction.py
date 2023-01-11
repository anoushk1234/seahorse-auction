# seahorse_auction
# Built with Seahorse v0.2.5

from seahorse.prelude import *


declare_id("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS")


class Auction(Account):
    ongoing: bool
    seller: Pubkey
    item_holder: Pubkey
    currency_holder: Pubkey
    currency: Pubkey
    refund_receiver: Pubkey
    bidder: Pubkey
    price: u64
    timed: bool
    go_live: i64
    end: i64


@instruction
def create_auction(
    auction: Empty[Auction],
    start_price: u64,
    payer: Signer,
    seller: UncheckedAccount,
    currency_holder: Empty[TokenAccount],
    item_holder: Empty[TokenAccount],
    currency: TokenMint,
    item: TokenMint,
    timed: bool,
    go_live: i64,
    end: i64,
):

    auction = auction.init(payer, ["auction", seller.key()])

    currency_holder.init(
        payer=payer,
        seeds=["currency_account"],
        mint=currency,
        authority=auction,
    )
    item_holder.init(payer=payer, seeds=["item_account"], mint=item, authority=auction)
    auction.ongoing = True
    auction.seller = seller.key()
    auction.item_holder = item_holder.key()
    auction.currency_holder = currency_holder.key()
    auction.currency = currency.key()
    auction.bidder = seller.key()
    auction.price = start_price
    auction.refund_receiver = seller.key()
    auction.timed = False
    if timed:
        auction.timed = timed
        assert go_live < end, "Start time exceeds end time"
        auction.go_live = go_live
        auction.end = end


@instruction
def bid(
    auction: Auction,
    price: u64,
    bidder: TokenAccount,
    authority: Signer,
    currency_holder: TokenAccount,
    refund_receiver: TokenAccount,
    clock: Clock,
):
    if auction.timed:
        assert clock.unix_timestamp() >= auction.go_live, "Auction hasn't started"
        assert clock.unix_timestamp() < auction.end, "Auction has ended"
    assert (
        price <= auction.price
    ), "Bid Price Too Low"  # bid shouldnt be less than previous bid
    assert (
        auction.currency_holder == currency_holder.key()
    ), "Unauthorized Currency Holder"  # ata validity check
    assert (
        refund_receiver.key() == auction.refund_receiver
    ), "Invalid Refund Receiver"  # ata validity check

    if auction.refund_receiver != auction.seller:
        currency_holder.transfer(
            auction, refund_receiver, refund_receiver.amount()
        )  # Previous bidder gets their bid refunded

    bidder.transfer(
        authority=authority, to=currency_holder, amount=price
    )  # tranfser bid the pda's token account

    auction.price = price  # update the price to the new bid
    auction.bidder = bidder.authority()  # set the new bidder


@instruction
def close_auction(
    auction: Auction,
    item_receiver: TokenAccount,
    item_holder: TokenAccount,
    currency_holder: TokenAccount,
    seller: Signer,
    seller_ata: TokenAccount,
    clock: Clock,
):
    if auction.timed:
        assert clock.unix_timestamp() >= auction.end, "Auction hasn't ended yet"
    assert item_holder.key() == auction.item_holder, "Unauthorized Item Holder"
    assert currency_holder.key() == auction.currency_holder, "Unauthorized Item Holder"
    assert auction.bidder == item_receiver.authority(), "Receiver not Bid Winner"
    assert (
        seller.key() == auction.seller and seller_ata.authority() == auction.seller
    ), "Signer not seller or unauthorized seller ata"

    item_holder.transfer(
        authority=auction, to=item_receiver, amount=item_holder.amount()
    )
    if currency_holder.amount() >= auction.price:
        currency_holder.transfer(
            authority=auction, to=seller_ata, amount=currency_holder.amount()
        )
    auction.ongoing = False
