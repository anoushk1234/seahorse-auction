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
    pub bidder: Pubkey,
    pub price: u64,
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
        let bidder = account.bidder.clone();
        let price = account.price;

        Mutable::new(LoadedAuction {
            __account__: account,
            __programs__: programs_map,
            ongoing,
            seller,
            item_holder,
            currency_holder,
            bidder,
            price,
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

        let bidder = loaded.bidder.clone();

        loaded.__account__.bidder = bidder;

        let price = loaded.price;

        loaded.__account__.price = price;
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
    pub bidder: Pubkey,
    pub price: u64,
}

pub fn bid_handler<'info>(
    mut auction: Mutable<LoadedAuction<'info, '_>>,
    mut price: u64,
    mut bidder: SeahorseAccount<'info, '_, TokenAccount>,
    mut authority: SeahorseSigner<'info, '_>,
    mut currency_holder: SeahorseAccount<'info, '_, TokenAccount>,
) -> () {
    if !(price <= auction.borrow().price) {
        panic!("Bid Price Too Low");
    }

    if !(auction.borrow().currency_holder == currency_holder.key()) {
        panic!("Unauthorized Currency Holder");
    }

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

    assign!(auction.borrow_mut().price, price);

    assign!(auction.borrow_mut().bidder, bidder.owner);
}

pub fn close_auction_handler<'info>(
    mut auction: Mutable<LoadedAuction<'info, '_>>,
    mut item_receiver: SeahorseAccount<'info, '_, TokenAccount>,
    mut item_holder: SeahorseAccount<'info, '_, TokenAccount>,
    mut currency_holder: SeahorseAccount<'info, '_, TokenAccount>,
    mut item_holder_auth: SeahorseSigner<'info, '_>,
    mut currency_holder_auth: SeahorseSigner<'info, '_>,
    mut currency_receiver: SeahorseAccount<'info, '_, TokenAccount>,
) -> () {
    if !(item_holder.key() == auction.borrow().item_holder) {
        panic!("Unauthorized Item Holder");
    }

    if !(currency_holder.key() == auction.borrow().currency_holder) {
        panic!("Unauthorized Item Holder");
    }

    if !(auction.borrow().bidder == item_receiver.owner) {
        panic!("Receiver not Owner");
    }

    token::transfer(
        CpiContext::new(
            item_holder.programs.get("token_program"),
            token::Transfer {
                from: item_holder.to_account_info(),
                authority: item_holder_auth.to_account_info(),
                to: item_receiver.to_account_info(),
            },
        ),
        item_holder.amount,
    )
    .unwrap();

    if currency_holder.amount >= auction.borrow().price {
        token::transfer(
            CpiContext::new(
                currency_holder.programs.get("token_program"),
                token::Transfer {
                    from: currency_holder.to_account_info(),
                    authority: currency_holder_auth.to_account_info(),
                    to: currency_receiver.to_account_info(),
                },
            ),
            auction.borrow().price,
        )
        .unwrap();
    }

    assign!(auction.borrow_mut().ongoing, false);
}

pub fn create_auction_handler<'info>(
    mut auction: Empty<Mutable<LoadedAuction<'info, '_>>>,
    mut start_price: u64,
    mut payer: SeahorseSigner<'info, '_>,
    mut seller: UncheckedAccount<'info>,
    mut currency_holder: SeahorseAccount<'info, '_, TokenAccount>,
    mut item_holder: SeahorseAccount<'info, '_, TokenAccount>,
) -> () {
    let mut auction = auction.account.clone();

    assign!(auction.borrow_mut().ongoing, true);

    assign!(auction.borrow_mut().seller, seller.key());

    assign!(auction.borrow_mut().item_holder, item_holder.key());

    assign!(auction.borrow_mut().currency_holder, currency_holder.key());

    assign!(auction.borrow_mut().bidder, seller.key());

    assign!(auction.borrow_mut().price, start_price);
}
