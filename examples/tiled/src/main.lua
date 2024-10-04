require("map-functions")

function love.load()
	loadMap()
end

function love.draw()
	drawMap()
end

function love.keypressed(k)
	if k == "escape" then
		love.event.quit(0)
	elseif k == "right" then
		if getMap() == "computer_lab" then
			setMap("tables")
			loadMap()
		else
			setMap("computer_lab")
			loadMap()
		end
	elseif k == "left" then
		if getMap() == "tables" then
			setMap("computer_lab")
			loadMap()
		else
			setMap("tables")
			loadMap()
		end
	end
end
