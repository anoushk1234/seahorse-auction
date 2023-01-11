#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use crate::{assign, id, index_assign, seahorse_util::*};
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::{cell::RefCell, rc::Rc};

#[account]
#[derive(Debug)]
pub struct Auction {
    pub ongoing: bool,
    pub seller: Pubkey,
    pub item_holder: Pubkey,
    pub currency_holder: Pubkey,
    pub currency: Pubkey,
    pub refund_receiver: Pubkey,
    pub bidder: Pubkey,
    pub price: u64,
    pub timed: bool,
    pub go_live: i64,
    pub end: i64,
}

impl<'info, 'entrypoint> Auction {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedAuction<'info, 'entrypoint>> {
        let ongoing = account.ongoing.clone();
        let seller = account.seller.clone();
        let item_holder = account.item_holder.clone();
        let currency_holder = account.currency_holder.clone();
        let currency = account.currency.clone();
        let refund_receiver = account.refund_receiver.clone();
        let bidder = account.bidder.clone();
        let price = account.price;
        let timed = account.timed.clone();
        let go_live = account.go_live;
        let end = account.end;

        Mutable::new(LoadedAuction {
            __account__: account,
            __programs__: programs_map,
            ongoing,
            seller,
            item_holder,
            currency_holder,
            currency,
            refund_receiver,
            bidder,
            price,
            timed,
            go_live,
            end,
        })
    }

    pub fn store(loaded: Mutable<LoadedAuction>) {
        let mut loaded = loaded.borrow_mut();
        let ongoing = loaded.ongoing.clone();

        loaded.__account__.ongoing = ongoing;

        let seller = loaded.seller.clone();

        loaded.__account__.seller = seller;

        let item_holder = loaded.item_holder.clone();

        loaded.__account__.item_holder = item_holder;

        let currency_holder = loaded.currency_holder.clone();

        loaded.__account__.currency_holder = currency_holder;

        let currency = loaded.currency.clone();

        loaded.__account__.currency = currency;

        let refund_receiver = loaded.refund_receiver.clone();

        loaded.__account__.refund_receiver = refund_receiver;

        let bidder = loaded.bidder.clone();

        loaded.__account__.bidder = bidder;

        let price = loaded.price;

        loaded.__account__.price = price;

        let timed = loaded.timed.clone();

        loaded.__account__.timed = timed;

        let go_live = loaded.go_live;

        loaded.__account__.go_live = go_live;

        let end = loaded.end;

        loaded.__account__.end = end;
    }
}

#[derive(Debug)]
pub struct LoadedAuction<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, Auction>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub ongoing: bool,
    pub seller: Pubkey,
    pub item_holder: Pubkey,
    pub currency_holder: Pubkey,
    pub currency: Pubkey,
    pub refund_receiver: Pubkey,
    pub bidder: Pubkey,
    pub price: u64,
    pub timed: bool,
    pub go_live: i64,
    pub end: i64,
}

pub fn bid_handler<'info>(
    mut auction: Mutable<LoadedAuction<'info, '_>>,
    mut price: u64,
    mut bidder: SeahorseAccount<'info, '_, TokenAccount>,
    mut authority: SeahorseSigner<'info, '_>,
    mut currency_holder: SeahorseAccount<'info, '_, TokenAccount>,
    mut refund_receiver: SeahorseAccount<'info, '_, TokenAccount>,
    mut clock: Sysvar<'info, Clock>,
) -> () {
    if auction.borrow().timed {
        if !(clock.unix_timestamp >= auction.borrow().go_live) {
            panic!("Auction hasn't started");
        }

        if !(clock.unix_timestamp < auction.borrow().end) {
            panic!("Auction has ended");
        }
    }

    if !(price > auction.borrow().price) {
        panic!("Bid Price Too Low");
    }

    if !(auction.borrow().currency_holder == currency_holder.key()) {
        panic!("Unauthorized Currency Holder");
    }

    if !(refund_receiver.key() == auction.borrow().refund_receiver) {
        panic!("Invalid Refund Receiver");
    }

    if auction.borrow().refund_receiver != currency_holder.key() {
        solana_program::msg!("{}", "valid refund receiver");

        token::transfer(
            CpiContext::new(
                currency_holder.programs.get("token_program"),
                token::Transfer {
                    from: currency_holder.to_account_info(),
                    authority: auction.borrow().__account__.to_account_info(),
                    to: refund_receiver.to_account_info(),
                },
            ),
            refund_receiver.amount,
        )
        .unwrap();

        solana_program::msg!("{}", "refund complete");
    }

    solana_program::msg!("{}", "Transfer from bidder to vault");

    token::transfer(
        CpiContext::new(
            bidder.programs.get("token_program"),
            token::Transfer {
                from: bidder.to_account_info(),
                authority: authority.to_account_info(),
                to: currency_holder.to_account_info(),
            },
        ),
        price,
    )
    .unwrap();

    solana_program::msg!("{}", "transfer complete");

    assign!(auction.borrow_mut().price, price);

    assign!(auction.borrow_mut().bidder, bidder.owner);

    assign!(auction.borrow_mut().refund_receiver, bidder.owner);
}

pub fn close_auction_handler<'info>(
    mut auction: Mutable<LoadedAuction<'info, '_>>,
    mut item_receiver: SeahorseAccount<'info, '_, TokenAccount>,
    mut item_holder: SeahorseAccount<'info, '_, TokenAccount>,
    mut currency_holder: SeahorseAccount<'info, '_, TokenAccount>,
    mut seller: SeahorseSigner<'info, '_>,
    mut seller_ata: SeahorseAccount<'info, '_, TokenAccount>,
    mut clock: Sysvar<'info, Clock>,
) -> () {
    if auction.borrow().timed {
        if !(clock.unix_timestamp >= auction.borrow().end) {
            panic!("Auction hasn't ended yet");
        }
    }

    if !(item_holder.key() == auction.borrow().item_holder) {
        panic!("Unauthorized Item Holder");
    }

    if !(currency_holder.key() == auction.borrow().currency_holder) {
        panic!("Unauthorized Item Holder");
    }

    if !(auction.borrow().bidder == item_receiver.owner) {
        panic!("Receiver not Bid Winner");
    }

    if !((seller.key() == auction.borrow().seller) && (seller_ata.owner == auction.borrow().seller))
    {
        panic!("Signer not seller or unauthorized seller ata");
    }

    solana_program::msg!("{}", "Transfer item to bid winner");

    token::transfer(
        CpiContext::new(
            item_holder.programs.get("token_program"),
            token::Transfer {
                from: item_holder.to_account_info(),
                authority: auction.borrow().__account__.to_account_info(),
                to: item_receiver.to_account_info(),
            },
        ),
        item_holder.amount,
    )
    .unwrap();

    solana_program::msg!("{}", "auction item transferred successfully");

    if currency_holder.amount >= auction.borrow().price {
        solana_program::msg!("{}", "Transferring bid payment to seller");

        token::transfer(
            CpiContext::new(
                currency_holder.programs.get("token_program"),
                token::Transfer {
                    from: currency_holder.to_account_info(),
                    authority: auction.borrow().__account__.to_account_info(),
                    to: seller_ata.to_account_info(),
                },
            ),
            currency_holder.amount,
        )
        .unwrap();

        solana_program::msg!("{}", "Bid transferred successfully");
    }

    assign!(auction.borrow_mut().ongoing, false);
}

pub fn create_auction_handler<'info>(
    mut auction: Empty<Mutable<LoadedAuction<'info, '_>>>,
    mut start_price: u64,
    mut payer: SeahorseSigner<'info, '_>,
    mut seller: UncheckedAccount<'info>,
    mut currency_holder: Empty<SeahorseAccount<'info, '_, TokenAccount>>,
    mut item_holder: Empty<SeahorseAccount<'info, '_, TokenAccount>>,
    mut currency: SeahorseAccount<'info, '_, Mint>,
    mut item: SeahorseAccount<'info, '_, Mint>,
    mut timed: bool,
    mut go_live: i64,
    mut end: i64,
) -> () {
    let mut auction = auction.account.clone();

    currency_holder.account.clone();

    item_holder.account.clone();

    assign!(auction.borrow_mut().ongoing, true);

    assign!(auction.borrow_mut().seller, seller.key());

    assign!(auction.borrow_mut().item_holder, item_holder.account.key());

    assign!(
        auction.borrow_mut().currency_holder,
        currency_holder.account.key()
    );

    assign!(auction.borrow_mut().currency, currency.key());

    assign!(auction.borrow_mut().bidder, seller.key());

    assign!(auction.borrow_mut().price, start_price);

    assign!(
        auction.borrow_mut().refund_receiver,
        currency_holder.account.key()
    );

    assign!(auction.borrow_mut().timed, false);

    if timed {
        assign!(auction.borrow_mut().timed, timed);

        if !(go_live < end) {
            panic!("Start time exceeds end time");
        }

        assign!(auction.borrow_mut().go_live, go_live);

        assign!(auction.borrow_mut().end, end);
    }
}

pub fn deposit_item_handler<'info>(
    mut seller_item_ata: SeahorseAccount<'info, '_, TokenAccount>,
    mut payer: SeahorseSigner<'info, '_>,
    mut item_holder: SeahorseAccount<'info, '_, TokenAccount>,
) -> () {
    token::transfer(
        CpiContext::new_with_signer(
            seller_item_ata.programs.get("token_program"),
            token::Transfer {
                from: seller_item_ata.to_account_info(),
                authority: payer.to_account_info(),
                to: item_holder.to_account_info(),
            },
            &[Mutable::new(vec![payer.key().as_ref()]).borrow().as_slice()],
        ),
        seller_item_ata.amount,
    )
    .unwrap();
}
