function init(plugin)
    plugin:newCommand{
        id = "ExportIm256",
        title = "Export To Im256",
        group = "file_export",
        onclick = function()
            if app.apiVersion < 1 then
                return app.alert("Your aseprite is too old. Please update your aseprite")
            end

            local sprite = app.activeSprite
            if sprite == nil then
                return app.alert("No Sprite...")
            end
            if sprite.colorMode ~= ColorMode.INDEXED then
                return app.alert("Sprite needs to be indexed")
            end

            local dlg = Dialog()
            dlg:file{
                id = "choose_save_loc",
                save = true,
                label = "choose location",
                title = "exporting to retro-blit",
                filename = "file.im256",
                filetypes = ".im256"
            }:button{
                id = "ok",
                text = "OK"
            }:button{
                id = "cancel",
                text = "Cancel"
            }:show()

            if dlg.data.ok then
                local file_name = dlg.data.choose_save_loc

                local palette = sprite.palettes[1]
                local frm = app.activeFrame

                local img = Image(sprite.spec)
                img:drawSprite(sprite, frm)

                local result_string = "IM"

                local ncolors = #palette
                local w = sprite.width
                local h = sprite.height

                result_string = result_string .. string.char(ncolors % 256)
                result_string = result_string .. string.char((ncolors / 256) % 256)
                result_string = result_string .. string.char(w % 256)
                result_string = result_string .. string.char((w / 256) % 256)
                result_string = result_string .. string.char(h % 256)
                result_string = result_string .. string.char((h / 256) % 256)

                for i = 0, ncolors - 1 do
                    local color = palette:getColor(i)
                    result_string = result_string .. string.char(color.red)
                    result_string = result_string .. string.char(color.green)
                    result_string = result_string .. string.char(color.blue)
                end

                for y = 0, h - 1 do
                    for x = 0, w - 1 do
                        px = img:getPixel(x, y)
                        result_string = result_string .. string.char(px)
                    end
                end

                local file = io.open(file_name, "w+b")
                io.output(file)
                io.write(result_string)
                io.close(file)
            end
        end
    }
end
