#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

pub mod dot;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use dot::program::*;
use std::{cell::RefCell, rc::Rc};

declare_id!("DPzpTr7kZupCWD98LaWNTtXAMhv3qKX7w9CdG7bo5acS");

pub mod seahorse_util {
    use super::*;

    #[cfg(feature = "pyth-sdk-solana")]
    pub use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};
    use std::{collections::HashMap, fmt::Debug, ops::Deref};

    pub struct Mutable<T>(Rc<RefCell<T>>);

    impl<T> Mutable<T> {
        pub fn new(obj: T) -> Self {
            Self(Rc::new(RefCell::new(obj)))
        }
    }

    impl<T> Clone for Mutable<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Deref for Mutable<T> {
        type Target = Rc<RefCell<T>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Debug> Debug for Mutable<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl<T: Default> Default for Mutable<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    impl<T: Clone> Mutable<Vec<T>> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    impl<T: Clone, const N: usize> Mutable<[T; N]> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    #[derive(Clone)]
    pub struct Empty<T: Clone> {
        pub account: T,
        pub bump: Option<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct ProgramsMap<'info>(pub HashMap<&'static str, AccountInfo<'info>>);

    impl<'info> ProgramsMap<'info> {
        pub fn get(&self, name: &'static str) -> AccountInfo<'info> {
            self.0.get(name).unwrap().clone()
        }
    }

    #[derive(Clone, Debug)]
    pub struct WithPrograms<'info, 'entrypoint, A> {
        pub account: &'entrypoint A,
        pub programs: &'entrypoint ProgramsMap<'info>,
    }

    impl<'info, 'entrypoint, A> Deref for WithPrograms<'info, 'entrypoint, A> {
        type Target = A;

        fn deref(&self) -> &Self::Target {
            &self.account
        }
    }

    pub type SeahorseAccount<'info, 'entrypoint, A> =
        WithPrograms<'info, 'entrypoint, Box<Account<'info, A>>>;

    pub type SeahorseSigner<'info, 'entrypoint> = WithPrograms<'info, 'entrypoint, Signer<'info>>;

    #[derive(Clone, Debug)]
    pub struct CpiAccount<'info> {
        #[doc = "CHECK: CpiAccounts temporarily store AccountInfos."]
        pub account_info: AccountInfo<'info>,
        pub is_writable: bool,
        pub is_signer: bool,
        pub seeds: Option<Vec<Vec<u8>>>,
    }

    #[macro_export]
    macro_rules! assign {
        ($ lval : expr , $ rval : expr) => {{
            let temp = $rval;

            $lval = temp;
        }};
    }

    #[macro_export]
    macro_rules! index_assign {
        ($ lval : expr , $ idx : expr , $ rval : expr) => {
            let temp_rval = $rval;
            let temp_idx = $idx;

            $lval[temp_idx] = temp_rval;
        };
    }
}

#[program]
mod seahorse_auction {
    use super::*;
    use seahorse_util::*;
    use std::collections::HashMap;

    #[derive(Accounts)]
    # [instruction (price : u64)]
    pub struct Bid<'info> {
        #[account(mut)]
        pub auction: Box<Account<'info, dot::program::Auction>>,
        #[account(mut)]
        pub bidder: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub authority: Signer<'info>,
        #[account(mut)]
        pub currency_holder: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub refund_receiver: Box<Account<'info, TokenAccount>>,
        #[account()]
        pub clock: Sysvar<'info, Clock>,
        pub token_program: Program<'info, Token>,
    }

    pub fn bid(ctx: Context<Bid>, price: u64) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let auction = dot::program::Auction::load(&mut ctx.accounts.auction, &programs_map);
        let bidder = SeahorseAccount {
            account: &ctx.accounts.bidder,
            programs: &programs_map,
        };

        let authority = SeahorseSigner {
            account: &ctx.accounts.authority,
            programs: &programs_map,
        };

        let currency_holder = SeahorseAccount {
            account: &ctx.accounts.currency_holder,
            programs: &programs_map,
        };

        let refund_receiver = SeahorseAccount {
            account: &ctx.accounts.refund_receiver,
            programs: &programs_map,
        };

        let clock = &ctx.accounts.clock.clone();

        bid_handler(
            auction.clone(),
            price,
            bidder.clone(),
            authority.clone(),
            currency_holder.clone(),
            refund_receiver.clone(),
            clock.clone(),
        );

        dot::program::Auction::store(auction);

        return Ok(());
    }

    #[derive(Accounts)]
    pub struct CloseAuction<'info> {
        #[account(mut)]
        pub auction: Box<Account<'info, dot::program::Auction>>,
        #[account(mut)]
        pub item_receiver: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub item_holder: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub currency_holder: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub seller: Signer<'info>,
        #[account(mut)]
        pub seller_ata: Box<Account<'info, TokenAccount>>,
        #[account()]
        pub clock: Sysvar<'info, Clock>,
        pub token_program: Program<'info, Token>,
    }

    pub fn close_auction(ctx: Context<CloseAuction>) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let auction = dot::program::Auction::load(&mut ctx.accounts.auction, &programs_map);
        let item_receiver = SeahorseAccount {
            account: &ctx.accounts.item_receiver,
            programs: &programs_map,
        };

        let item_holder = SeahorseAccount {
            account: &ctx.accounts.item_holder,
            programs: &programs_map,
        };

        let currency_holder = SeahorseAccount {
            account: &ctx.accounts.currency_holder,
            programs: &programs_map,
        };

        let seller = SeahorseSigner {
            account: &ctx.accounts.seller,
            programs: &programs_map,
        };

        let seller_ata = SeahorseAccount {
            account: &ctx.accounts.seller_ata,
            programs: &programs_map,
        };

        let clock = &ctx.accounts.clock.clone();

        close_auction_handler(
            auction.clone(),
            item_receiver.clone(),
            item_holder.clone(),
            currency_holder.clone(),
            seller.clone(),
            seller_ata.clone(),
            clock.clone(),
        );

        dot::program::Auction::store(auction);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (start_price : u64 , timed : bool , go_live : i64 , end : i64)]
    pub struct CreateAuction<'info> {
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: Auction > () + 8 , payer = payer , seeds = ["auction" . as_bytes () . as_ref () , seller . key () . as_ref ()] , bump)]
        pub auction: Box<Account<'info, dot::program::Auction>>,
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        #[doc = "CHECK: This account is unchecked."]
        pub seller: UncheckedAccount<'info>,
        # [account (init , payer = payer , associated_token :: mint = currency , associated_token :: authority = auction)]
        pub currency_holder: Box<Account<'info, TokenAccount>>,
        # [account (init , payer = payer , associated_token :: mint = item , associated_token :: authority = auction)]
        pub item_holder: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub currency: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub item: Box<Account<'info, Mint>>,
        pub associated_token_program: Program<'info, AssociatedToken>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
    }

    pub fn create_auction(
        ctx: Context<CreateAuction>,
        start_price: u64,
        timed: bool,
        go_live: i64,
        end: i64,
    ) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "associated_token_program",
            ctx.accounts.associated_token_program.to_account_info(),
        );

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let auction = Empty {
            account: dot::program::Auction::load(&mut ctx.accounts.auction, &programs_map),
            bump: ctx.bumps.get("auction").map(|bump| *bump),
        };

        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let seller = &ctx.accounts.seller.clone();
        let currency_holder = Empty {
            account: SeahorseAccount {
                account: &ctx.accounts.currency_holder,
                programs: &programs_map,
            },
            bump: ctx.bumps.get("currency_holder").map(|bump| *bump),
        };

        let item_holder = Empty {
            account: SeahorseAccount {
                account: &ctx.accounts.item_holder,
                programs: &programs_map,
            },
            bump: ctx.bumps.get("item_holder").map(|bump| *bump),
        };

        let currency = SeahorseAccount {
            account: &ctx.accounts.currency,
            programs: &programs_map,
        };

        let item = SeahorseAccount {
            account: &ctx.accounts.item,
            programs: &programs_map,
        };

        create_auction_handler(
            auction.clone(),
            start_price,
            payer.clone(),
            seller.clone(),
            currency_holder.clone(),
            item_holder.clone(),
            currency.clone(),
            item.clone(),
            timed,
            go_live,
            end,
        );

        dot::program::Auction::store(auction.account);

        return Ok(());
    }
}
