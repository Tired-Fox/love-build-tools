local tileString = [[
#########################
#                       #
#   [] # []          ^  #
#   () # ()          @  #
#      #                #
#   #######             #
#      #                #
#   [] # []             #
#   () # ()             #
#      #                #
#   #######             #
#      #                #
#   [] # []          ^  #
#   () # ()          @  #
#                       #
#########################
]]

local quadInfo = {
	{ " ", 0, 0 }, -- floor
	{ "[", 1, 0 }, -- table top left
	{ "]", 2, 0 }, -- table top right
	{ "(", 1, 1 }, -- table bottom left
	{ ")", 2, 1 }, -- table bottom right
	{ "#", 0, 1 }, -- Bricks
	{ "^", 3, 0 }, -- Plant top
	{ "@", 3, 1 }, -- Plant bottom
}

newMap(32, 32, "/assets/lab.png", tileString, quadInfo)
