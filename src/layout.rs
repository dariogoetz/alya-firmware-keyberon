use keyberon::action::{k, m, Action::*, HoldTapAction, HoldTapConfig};
use keyberon::key_code::KeyCode::*;

type Action = keyberon::action::Action<()>;

static DLAYER: Action = Action::DefaultLayer(0);
static QWERTZLAYER: Action = Action::DefaultLayer(4);

const TIMEOUT: u16 = 200;

const SHIFT_SP: Action = HoldTap(&HoldTapAction {
    timeout: TIMEOUT,
    tap_hold_interval: 200,
    config: HoldTapConfig::Default,
    hold: k(LShift),
    tap: k(Space),
});

const CTRL_TAB: Action = HoldTap(&HoldTapAction {
    timeout: TIMEOUT,
    tap_hold_interval: 200,
    config: HoldTapConfig::Default,
    hold: k(LCtrl),
    tap: k(Tab),
});

const ALT_ENT: Action = HoldTap(&HoldTapAction {
    timeout: TIMEOUT,
    tap_hold_interval: 200,
    config: HoldTapConfig::Default,
    hold: k(LAlt),
    tap: k(Enter),
});

const PPN: Action = HoldTap(&HoldTapAction {
    timeout: TIMEOUT,
    tap_hold_interval: 200,
    config: HoldTapConfig::Default,
    hold: k(MediaNextSong),
    tap: k(MediaPlayPause),
});

macro_rules! s {
    ($k:ident) => {
        m(&[LShift, $k].as_slice())
    };
}
macro_rules! a {
    ($k:ident) => {
        m(&&[RAlt, $k].as_slice())
    };
}

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers<7, 10, 5, ()> = keyberon::layout::layout! {
    {
        // left half
        [ n     n     1     2     3     4     5],
        [ n     J     Y     Z     U     A     Q],
        [(1)    n     C     S     I     E     O],
        [LGui   V     X LBracket Quote  ;     n],
        [ t     t     t     t    (2) LShift {CTRL_TAB}],

        // right half
        [ 6     7     8     9     0     n    n],
        [ P     B     M     L     F     -    n],
        [ D     T     N     R     H     n   (1)],
        [ n     W     G     ,     .     K   LGui],
        [ {ALT_ENT} {SHIFT_SP} (2) t t  t    t],
    }{
        // left half
        [ t   t           t             t                t            t            t],
        [ t   t         {a!(E)}     {s!(Slash)}       {a!(Kb8)}     {a!(Kb9)}      Grave],
        [ t   n       {a!(Minus)}    {s!(Kb7)}        {a!(Kb7)}     {a!(Kb0)}  {s!(RBracket)}],
        [ t NonUsHash   {s!(Kb4)} {a!(NonUsBslash)} {a!(RBracket)} {s!(Equal)}      t],
        [ t   t           t             t               (3)           t             t],

        // right half
        [ t              t            t               t            t            t        t],
        [ {s!(Kb1)}  NonUsBslash {s!(NonUsBslash)} {s!(Kb0)}   {s!(Kb6)}     {a!(Q)}     t],
        [ {s!(Minus)} {s!(Kb8)}   {s!(Kb9)}         Slash      {s!(Dot)}        n        t],
        [ n           RBracket    {s!(Kb5)}        {s!(Kb2)} {s!(NonUsHash)} {s!(Comma)} t],
        [ t              t           (3)              t            t            t        t],
    }{
        // left half
        [ t  t  t      t      t    t      t],
        [ t  t  PgUp   BSpace Up   Delete PgDown],
        [(3) n  Home   Left   Down Right  End],
        [ t  t  Escape Tab    n    Enter  n],
        [ t  t  t      t      t    t      t],

        // right half
        [ t   t   t   t   t        t              t],
        [ n   Kb7 Kb8 Kb9 RBracket Slash          t],
        [ n   Kb4 Kb5 Kb6 Dot      n         {s!(RBracket)}],
        [ n   Kb0 Kb1 Kb2 Kb3      Comma      {s!(Kb7)}],
        [ t   t   t   t   t        t              t],
    }{
        // left half
        [ t            t     t     t     t      t     t],
        [{Custom(())}  n     n     n     n    VolUp   n],
        [t             n     n     n     n   {PPN}    n],
        [t             n     n     n     n   VolDown  n],
        [t             t     t     t     t     t      t],

        // right half
        [ t   t   t   t   t     t       t],
        [F12  F7  F8  F9  n     n  {Custom(())}],
        [F11  F4  F5  F6  n     n      t],
        [n    F10 F1  F2  F3    n      t],
        [t    t {QWERTZLAYER} t t t    t],
    }{
        // left half
        [ t      t    t   t   t   t   t],
        [ Tab    n    Q   W   E   R   T],
        [ LCtrl  n    A   S   D   F   G],
        [ LShift Z    X   C   V   B   n],
        [ n      n    n   n LGui LCtrl LAlt],

        // right half
        [ t   t   t   t   t    t      t],
        [ Y   U   I   O   P   BSpace  t],
        [ H   J   K   L   ;    n   Quote],
        [ n   N   M   ,   .   /   Escape ],
        [ Enter Space {DLAYER} n n n n ],
     }
};
