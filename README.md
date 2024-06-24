# ResBox
(name subject to change)

A client for the social features of [Resonite](https://store.steampowered.com/app/2519830/Resonite/), inspired by (a direct, shameless, and attempted pixel-perfect copy of) the 2015 Xbox One guide.

**I'm not too keen on accepting contributions for this.** I enjoy working on this project, in almost every aspect. This is only here for easier sharing to the friend(s) that want to use it, and to help diagnose bugs I rant about on discord. I'm not going to stop you from submitting fixes but please keep in mind that I enjoy working on this, so large contibutions may hamper that.

# Features
- Current:
  - Profile viewing
  - User searching
  - Contacts viewing
  - message history viewing
- Planned:
  - Contact status viewing
  - Messaging
  - Friend requests and blocking
  - Session browser
- Unplanned:
  - Inventory viewing and management

# Building
This app relies on three Microsoft fonts, I don't like checking in content I don't own to VCS, along with being fuzzy on licensing, so you'll need to provide them yourself. Luckily they're included with Windows 10. (I do not know if they're present in Windows 11, nor do I care, nor will I fix that problem for you.)
- segoeui.ttf <- `C:\Windows\Fonts\segoeui.ttf` - General UI font
- segmdl2.ttf <- `C:\Windows\Fonts\segmdl2.ttf` - Icons
- segoe_slboot.ttf <- `C:\Windows\Boot\Fonts\segoe_slboot.ttf` - Loading spinner

Put those three files in the root dir (next to `src/`) and it should just work, it's rust idk how hard could it be