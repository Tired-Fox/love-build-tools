local tileW, tileH, tileset, quads, tileTable

--- @enum (key) Maps
local maps = {
	computer_lab = { "Computer Lab", "maps/computer-lab.lua" },
	tables = { "Tables", "maps/tables.lua" },
}

--- @type Maps
local currentMap = "computer_lab"

function newMap(tileWidth, tileHeight, tilesetPath, tileString, quadInfo)
	tileW = tileWidth
	tileH = tileHeight
	tileset = love.graphics.newImage(tilesetPath)

	local tilesetW, tilesetH = tileset:getWidth(), tileset:getHeight()

	quads = {}

	for _, info in ipairs(quadInfo) do
		-- info[1] = the character, info[2] = x, info[3] = y
		quads[info[1]] = love.graphics.newQuad(info[2] * tileW, info[3] * tileH, tileW, tileH, tilesetW, tilesetH)
	end

	tileTable = {}

	local width = #(tileString:match("[^\n]+"))

	for x = 1, width, 1 do
		tileTable[x] = {}
	end

	local x, y = 1, 1
	for row in tileString:gmatch("[^\n]+") do
		assert(
			#row == width,
			"Map is not aligned: width of row "
				.. tostring(y)
				.. " should be "
				.. tostring(width)
				.. ", but it is "
				.. tostring(#row)
		)
		x = 1
		for tile in row:gmatch(".") do
			tileTable[x][y] = tile
			x = x + 1
		end
		y = y + 1
	end
end

function loadMap()
	local map = maps[currentMap]
	love.window.setTitle(map[1])
	love.filesystem.load(map[2])() -- attention! extra parenthesis
end

--- @param map Maps
function setMap(map)
	if maps[map] == nil then
		error(map .. " is not a valid map")
	end
	currentMap = map
end

--- @return Maps name of the currently loaded map
function getMap()
	return currentMap
end

function drawMap()
	for columnIndex, column in ipairs(tileTable) do
		for rowIndex, char in ipairs(column) do
			local x, y = (columnIndex - 1) * tileW, (rowIndex - 1) * tileH
			love.graphics.draw(tileset, quads[char], x, y)
		end
	end
end
