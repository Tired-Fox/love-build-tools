local tileString = [[
#########################
#                #      #
#  L[]R   L[]R   # L[]R #
#  L()R   L()R   # L()R #
#                #      #
#                ###  ###
#  L[]R   L[]R          #
#  L()R   L()R    L[]R  #
#                 L()R  #
#                       #
#  L[]R   L[]R          #
#  L()R   L()R   ###  ###
#                #LL  RR#
#                #LL  RR#
#  L[]R   L[]R   #LL  RR#
#  L()R   L()R   #LL  RR#
#                #LL  RR#
#########################
]]

local quadInfo = {
	{ " ", 0, 0 }, -- floor
	{ "[", 1, 0 }, -- table top left
	{ "]", 2, 0 }, -- table top right
	{ "(", 1, 1 }, -- table bottom left
	{ ")", 2, 1 }, -- table bottom right
	{ "L", 0, 1 }, -- chair on the left
	{ "R", 3, 1 }, -- chair on the right
	{ "#", 3, 0 }, -- bricks
}

newMap(32, 32, "/assets/resto.png", tileString, quadInfo)
