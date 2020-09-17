import {
    Account,
    add_filter_category,
    clear_filter_categories,
    CategoryType,
    delete_account_order,
    Filter,
    TransactionState,
    set_account_order_date,
    set_account_order_description,
    set_account_order_amount,
    set_account_order_resource,
    set_account_order_tags,
    set_account_order_state,
    get_account_categories,
    get_account_filtered_orders,
    get_filter_category_state,
    load_account_data,
    remove_filter_category,
    serialize_account_as_yaml,
    set_filter_categories,
    sum_filtered_orders,
    toggle_filter_category,
    toggle_account_order_visibility,
    ItemSelector,
    VisibilityFilter,
} from "money"

import {
    clearTableRows,
    getEnumStrings,
    enumIndexToString,
    enumStringToIndex,
    removeChildNodeById,
    removeChildNodesByTagName,
    dateString,
    firstDayCurrentMonthString,
    lastDayCurrentMonthString,
} from "./modules/utils.js"

// Singleton
const account = new Account()
const filter = new Filter()
// Tags management
const tagsCluster = document.getElementById("tags-manager")
const inputTag = document.getElementById("input-tag")
const tagsList = document.getElementById("tag-list")
const addTag = document.getElementById("add-tag")
const removeTag = document.getElementById("remove-tag")
const tagsFilterButton = document.getElementById("tags-filter-switch")
var tagsHideFilter = true
// Resources management
const resourcesCluster = document.getElementById("resources-manager")
const inputResource = document.getElementById("input-resource")
const resourcesList = document.getElementById("resource-list")
const addResource = document.getElementById("add-resource")
const removeResource = document.getElementById("remove-resource")
const resourcesFilterButton = document.getElementById("resources-filter-switch")
var resourcesHideFilter = true
// Orders
const addOrder = document.getElementById("add-order")
const ordersTable = document.getElementById("orders")
const statesCluster = document.getElementById("states-manager")
const dateCluster = document.getElementById("date-manager")
const dateFilterButton = document.getElementById("date-filter-switch")
var dateHideFilter = true
const allRadioButton = document.getElementById("all-radio")
const activeRadioButton = document.getElementById("active-radio")
const removedRadioButton = document.getElementById("removed-radio")
// File management
const loadData = document.getElementById("load-data")
const downloadData = document.getElementById("download-data")
// Sum
const sum = document.getElementById("sum-canvas")
// Enumerations
const transactionStateEnum = getEnumStrings(TransactionState)
const itemSelectorEnum = getEnumStrings(ItemSelector)
const visibilityEnum = getEnumStrings(VisibilityFilter)
const categoryTypeEnum = getEnumStrings(CategoryType)
const resourceCategoryType = enumStringToIndex(categoryTypeEnum, "Resource")
const tagCategoryType = enumStringToIndex(categoryTypeEnum, "Tag")

const addCategorySelectButton = (node, list) => {
    if (list.length != 0) {
        var button = document.createElement("input")
        button.type = "button"
        button.value = "deselect all"
        button.id = "select-button"

        button.addEventListener('click', () => {
            var categoryType = undefined
            switch (node) {
                case resourcesCluster:
                    categoryType = resourceCategoryType
                    break
                case tagsCluster:
                    categoryType = tagCategoryType
                    break
            }

            if (categoryType != undefined) {
                if (button.value == "deselect all") {
                    button.value = "select all"
                    const selectedIndex = enumStringToIndex(itemSelectorEnum, "Selected")

                    list.forEach(function(item) {
                        if (get_filter_category_state(filter, categoryType, item) == selectedIndex) {
                            toggle_filter_category(filter, categoryType, item)
                        }
                    })
                } else {
                    button.value = "deselect all"
                    const discardedIndex = enumStringToIndex(itemSelectorEnum, "Discarded")

                    list.forEach(function(item) {
                        if (get_filter_category_state(filter, categoryType, item) == discardedIndex) {
                            toggle_filter_category(filter, categoryType, item)
                        }
                    })
                }
                requestAnimationFrame(render)
            } else {
                console.error("Invalid node: ", node)
            }
        })

        node.appendChild(button)
    }
}

const addCategoryCheckboxes  = (node, list) => {
    // Add one for each item
    list.forEach(function(item) {
        var div = document.createElement("div")
        var label = document.createElement("label")
        var checkbox = document.createElement("input")
        label.appendChild(document.createTextNode(item))
        checkbox.type = "checkbox"

        var categoryType = undefined
        switch (node) {
            case resourcesCluster:
                categoryType = resourceCategoryType
                break
            case tagsCluster:
                categoryType = tagCategoryType
                break
        }

        if (categoryType != undefined) {
            // Initialize the checkbox with current value (should always be 'Selected')
            const string = enumIndexToString(itemSelectorEnum, get_filter_category_state(filter, categoryType, item))
            if (string == "Selected") {
                checkbox.checked = true
            } else if (string == "Discarded") {
                checkbox.checked = false
            } else {
                console.error("Unknown selector", string)
            }

            checkbox.addEventListener('change', function() {
                if (toggle_filter_category(filter, categoryType, item) != undefined) {
                    requestAnimationFrame(render)
                }
                else {
                    console.error("Unknown selector", item)
                }
            }, false)

            div.appendChild(checkbox)
            div.appendChild(label)
            node.appendChild(div)
        }
    })
}

const setStatesCheckboxes = () => {
    transactionStateEnum.forEach(function(item) {
        var div = document.createElement("div")
        var label = document.createElement("label")
        var checkbox = document.createElement("input")
        label.appendChild(document.createTextNode(item[0]))
        checkbox.type = "checkbox"

        const index = enumStringToIndex(transactionStateEnum, item[0])

        // Initialize the checkbox with current value (should always be 'Selected')
        const string = enumIndexToString(itemSelectorEnum, filter.get_state(index))
        if (string == "Selected") {
            checkbox.checked = true
        } else if (string == "Discarded") {
            checkbox.checked = false
        } else {
            console.error("Unknown selector", string)
        }

        checkbox.addEventListener('change', function() {
            filter.toggle_state(index)
            requestAnimationFrame(render)
        }, false)

        div.appendChild(checkbox)
        div.appendChild(label)
        statesCluster.appendChild(div)
    })
}

const refreshCategoryCombobox  = (node, list) => {
    // Remove all options
    while (node.firstChild) {
        node.removeChild(node.lastChild)
    }
    // Add options
    list.forEach(function(item) {
        var option = document.createElement('option')
        option.value = item
        node.appendChild(option)
    })
}

const refreshCategoryList = (type) => {
    var cluster
    var combobox
    var filter_state
    var categoryType = undefined
    var error = false
    switch (type) {
        case "Tag":
            cluster = tagsCluster
            combobox = tagsList
            categoryType = tagCategoryType
            filter_state = tagsHideFilter
            break;
        case "Resource":
            cluster = resourcesCluster
            combobox = resourcesList
            categoryType = resourceCategoryType
            filter_state = resourcesHideFilter
            break;
        default:
            error = true
            console.error("Unknown category type: ", type)
    }

    if (error == false) {
        const list = get_account_categories(account, categoryType)
        if (list != undefined) {
            // Combobox
            refreshCategoryCombobox(combobox, list)
            // Checkboxes
            if (!filter_state) {
                removeChildNodesByTagName(cluster, "DIV")
                addCategoryCheckboxes(cluster, list)
            }
        } else {
            console.error("Unable to get " + type + " categories")
        }
    }
}

const addDateRangeInputs = () => {
    var div_start = document.createElement("div")
    var div_stop = document.createElement("div")
    var label_start = document.createElement("label")
    var label_stop = document.createElement("label")
    label_start.appendChild(document.createTextNode("Start"))
    label_stop.appendChild(document.createTextNode("Stop"))
    var begin = document.createElement("input")
    begin.type = "text"
    var end = document.createElement("input")
    end.type = "text"

    begin.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (!filter.set_date_beginning(begin.value)) {
                begin.value = ""
            }
            requestAnimationFrame(render)
        }
    })

    begin.addEventListener('click', () => {
        begin.value = firstDayCurrentMonthString()
    })

    end.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (!filter.set_date_end(end.value)) {
                end.value = ""
            }
            requestAnimationFrame(render)
        }
    })

    end.addEventListener('click', () => {
        end.value = lastDayCurrentMonthString()
    })

    div_start.appendChild(begin)
    div_start.appendChild(label_start)
    dateCluster.appendChild(div_start)
    div_start.appendChild(end)
    div_start.appendChild(label_stop)
    dateCluster.appendChild(div_stop)
}

const addOrderRow = (obj) => {
    var row = ordersTable.insertRow()
    const resourceList = get_account_categories(account, resourceCategoryType)
    const tagList = get_account_categories(account, tagCategoryType)

    // Date
    var date = document.createElement("input")
    date.type = "text"
    date.value = obj.order.date
    date.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (set_account_order_date(account, obj.id, date.value)) {
                console.log("Order " + obj.id + " date: " + date.value)
                requestAnimationFrame(render)
            } else {
                alert("Expected date format:  YY-MM-DD")
            }
        }
    })
    date.addEventListener('click', () => {
        date.value = dateString(new Date())
    })
    row.insertCell().appendChild(date)

    // Description node
    var description = document.createElement("input")
    description.type = "text"
    description.value = obj.order.description
    description.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (set_account_order_description(account, obj.id, description.value)) {
                console.log("Order " + obj.id + " description: " + description.value)
                requestAnimationFrame(render)
            }
        }
    })
    row.insertCell().appendChild(description)

    // Amount node
    var amount = document.createElement("input")
    amount.type = "text"
    amount.value = obj.order.amount.toFixed(2) + "€"
    amount.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (amount.value == "") {
                amount.value = "0.0"
            }

            const float = parseFloat(amount.value)
            if (set_account_order_amount(account, obj.id, float)) {
                console.log("Order " + obj.id + " amount: " + float.toFixed(2))
                requestAnimationFrame(render)
            }
        }
    })
    amount.addEventListener('click', event => {
        amount.value = ""
    })
    row.insertCell().appendChild(amount)

    // Resource node
    var resource = document.createElement("select")
    var empty_option = document.createElement("option")
    empty_option.value = "-"
    empty_option.text = "-"
    empty_option.disabled = true
    resource.appendChild(empty_option)
    resourceList.forEach(function(item) {
        var option = document.createElement("option")
        option.value = item
        option.text = item
        resource.appendChild(option)
    })
    if (obj.order.resource != null) {
        resource.value = obj.order.resource
    } else {
        resource.value = "-"
    }
    resource.addEventListener('change', function() {
        if (set_account_order_resource(account, obj.id, this.value)) {
            console.log("Order " + obj.id + " resource: " + this.value)
            requestAnimationFrame(render)
        }
    }, false)
    row.insertCell().appendChild(resource)

    // Tags
    var tags = document.createElement("select")
    tags.multiple = true
    tagList.forEach(function(item) {
        var option = document.createElement("option")
        option.value = item
        option.text = item

        obj.order.tags.forEach(function(tag) {
            if (item == tag) {
                option.selected = true
            }
        })

        tags.appendChild(option)
    })
    tags.addEventListener('change', function() {
        const selectedValues = [...this.options]
                     .filter((x) => x.selected)
                     .map((x)=>x.value)

        if (set_account_order_tags(account, obj.id, selectedValues)) {
            console.log("Order " + obj.id + " tags: " + selectedValues)
            requestAnimationFrame(render)
        } else {
            console.error("Unknown ", obj.id, selectedValues)
        }
    }, false)
    row.insertCell().appendChild(tags)

    // State
    var state = document.createElement("select")
    for (const entry of transactionStateEnum) {
        var option = document.createElement("option")
        option.text = entry[0]
        option.value = entry[0]
        state.appendChild(option)
    }
    state.value = obj.order.state
    state.addEventListener('change', function() {
        const index = enumStringToIndex(transactionStateEnum, this.value)
        if (set_account_order_state(account, obj.id, index)) {
            console.log("Order " + obj.id + " state: " + this.value)
            requestAnimationFrame(render)
        }
    }, false)
    row.insertCell().appendChild(state)

    // Remove button node
    if (obj.order.visible) {
        var remove_button = document.createElement("input")
        remove_button.type = "button"
        remove_button.value = "remove"
        remove_button.addEventListener('click', event => {
            if (toggle_account_order_visibility(account, obj.id)) {
                console.log("Order " + obj.id + ": removed!")
            }
            requestAnimationFrame(render)
        })
        row.insertCell().appendChild(remove_button)
    } else {
        var restore_button = document.createElement("input")
        var delete_button = document.createElement("input")
        restore_button.type = "button"
        restore_button.value = "restore"
        delete_button.type = "button"
        delete_button.value = "delete"
        restore_button.addEventListener('click', event => {
            if (toggle_account_order_visibility(account, obj.id)) {
                console.log("Order " + obj.id + ": restored!")
            }
            requestAnimationFrame(render)
        })
        delete_button.addEventListener('click', event => {
            if (confirm("Are you sure?")) {
                if (delete_account_order(account, obj.id)) {
                    console.log("Order " + obj.id + ": deleted!")
                }
                requestAnimationFrame(render)
            }
        })
        row.insertCell().appendChild(restore_button)
        row.insertCell().appendChild(delete_button)
    }
}

addTag.addEventListener("click", event => {
    if (account.add_tag(inputTag.value) == undefined) {
        if (!tagsHideFilter) {
            add_filter_category(filter, tagCategoryType, inputTag.value)
        }
        refreshCategoryList("Tag")
        inputTag.value = ""
        requestAnimationFrame(render)
    }
})

removeTag.addEventListener("click", event => {
    if (account.remove_tag(inputTag.value) == undefined) {
        if (!tagsHideFilter) {
            remove_filter_category(filter, tagCategoryType, inputTag.value)
        }
        refreshCategoryList("Tag")
        inputTag.value = ""
        requestAnimationFrame(render)
    }
})

tagsFilterButton.addEventListener("click", event => {
    const categoryType = tagCategoryType
    if (tagsHideFilter) {
        tagsFilterButton.textContent = "disable filter"

        const tags = get_account_categories(account, categoryType)
        set_filter_categories(filter, categoryType, tags)
        addCategorySelectButton(tagsCluster, tags)
        addCategoryCheckboxes(tagsCluster, tags)
    } else {
        tagsFilterButton.textContent = "enable filter"
        removeChildNodesByTagName(tagsCluster, "DIV")
        removeChildNodeById(tagsCluster, "select-button")
        clear_filter_categories(filter, categoryType)
        requestAnimationFrame(render)
    }

    // Toggle
    tagsHideFilter = !tagsHideFilter
})

addResource.addEventListener("click", event => {
    if (account.add_resource(inputResource.value) == undefined) {
        if (!resourcesHideFilter) {
            add_filter_category(filter, resourceCategoryType, inputResource.value)
        }
        refreshCategoryList("Resource")
        inputResource.value = ""
        requestAnimationFrame(render)
    }
})

removeResource.addEventListener("click", event => {
    if (account.remove_resource(inputResource.value) == undefined) {
        if (!resourcesHideFilter) {
            remove_filter_category(filter, resourceCategoryType, inputResource.value)
        }
        refreshCategoryList("Resource")
        inputResource.value = ""
        requestAnimationFrame(render)
    }
})

resourcesFilterButton.addEventListener("click", event => {
    const categoryType = resourceCategoryType
    if (resourcesHideFilter) {
        resourcesFilterButton.textContent = "disable filter"

        const resources = get_account_categories(account, categoryType)
        set_filter_categories(filter, categoryType, resources)
        addCategorySelectButton(resourcesCluster, resources)
        addCategoryCheckboxes(resourcesCluster, resources)
    } else {
        resourcesFilterButton.textContent = "enable filter"
        removeChildNodesByTagName(resourcesCluster, "DIV")
        removeChildNodeById(resourcesCluster, "select-button")
        clear_filter_categories(filter, categoryType)
        requestAnimationFrame(render)
    }

    // Toggle
    resourcesHideFilter = !resourcesHideFilter
})

dateFilterButton.addEventListener("click", event => {
    if (dateHideFilter) {
        dateFilterButton.textContent = "disable filter"
        addDateRangeInputs()
    } else {
        dateFilterButton.textContent = "enable filter"
        removeChildNodesByTagName(dateCluster, "DIV")
        filter.disable_date_option()
        requestAnimationFrame(render)
    }

    // Toggle
    dateHideFilter = !dateHideFilter
})

allRadioButton.addEventListener("click", event => {
    filter.visibility = enumStringToIndex(visibilityEnum, "VisibilityIgnored")
    requestAnimationFrame(render)
})
activeRadioButton.addEventListener("click", event => {
    filter.visibility = enumStringToIndex(visibilityEnum, "VisibleOnly")
    requestAnimationFrame(render)
})
removedRadioButton.addEventListener("click", event => {
    filter.visibility = enumStringToIndex(visibilityEnum, "HiddenOnly")
    requestAnimationFrame(render)
})

addOrder.addEventListener("click", event => {
    account.add_order()
    requestAnimationFrame(render)
})

// Load YAML file
loadData.addEventListener("change", function() {
    var file = this.files[0]
    var reader = new FileReader()

    reader.readAsText(file,'UTF-8')

    reader.onload = readerEvent => {
        var content = readerEvent.target.result

        if (load_account_data(account, content)) {
            console.log("File '" + file + "' loaded!")
            requestAnimationFrame(render)
        }
    }
}, false)

// Write YAML file
downloadData.addEventListener("click", event => {
    function download(filename, text) {
        var element = document.createElement('a')
        element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(text))
        element.setAttribute('download', filename)

        element.style.display = 'none'
        document.body.appendChild(element)

        element.click()

        document.body.removeChild(element)
    }

    const filename = prompt("Please enter file name:", "account-data.yml")

    if (filename != null) {
        const data = serialize_account_as_yaml(account)
        download(filename, data)
    }
})

const render = () => {
    console.log("Render!")
    // Configuration
    // To be removed!?
    refreshCategoryList("Tag")
    refreshCategoryList("Resource")

    // Order visibility
    switch (filter.visibility) {
        case enumStringToIndex(visibilityEnum, "VisibilityIgnored"):
            allRadioButton.checked = true
            break;
        case enumStringToIndex(visibilityEnum, "VisibleOnly"):
            activeRadioButton.checked = true
            break;
        case enumStringToIndex(visibilityEnum, "HiddenOnly"):
            removedRadioButton.checked = true
            break;
    }

    // Clear table rows
    clearTableRows(ordersTable)

    const orders = get_account_filtered_orders(account, filter)
    if (!orders.length == 0) {
        // Add table rows
        orders.forEach(function(item) {
            var obj = JSON.parse(item)
            addOrderRow(obj)
        })
    }

    // Sum
    sum.textContent = sum_filtered_orders(account, filter).toFixed(2) + '€'
}

setStatesCheckboxes()
requestAnimationFrame(render)
