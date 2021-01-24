# Gbar
demenu/rofi alternative for wayland (using gtk)

# Sway config example
```config
set $menu dmenu_path | gbar_client | xargs swaymsg exec
for_window [app_id="gbar"] floating enable
exec_always gbar
```
# How it looks
<img src="./gbar.png" width="70%" height="70%">
