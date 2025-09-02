# To-do list

<dt><b>Flag:</b> <code>--use-done</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_USE_DONE</code></dt>
<dt><b>Conf:</b> <code>use_done</code></dt>
<dd>
Determines whether projects, contexts, and tags from completed tasks should be included in the lists of available projects, contexts, and tags.  

- **Default:** `false`
</dd>
<br>

<dt><b>Flag:</b> <code>--pending-sort</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PENDING_SORT</code></dt>
<dt><b>Conf:</b> <code>pending_sort</code></dt>
<dd>
Specifies the sorting option for pending tasks.  

- **Possible Values (Flag):** `none`, `reverse`, `priority`, `alphanumeric`, `alphanumeric-reverse`  
- **Possible Values (Config):** `None`, `Reverse`, `Priority`, `Alphanumeric`, `AlphanumericReverse`  
- **Default:** `None`

**Sorting Options**:

- **None:** No specific sorting; tasks appear in the order they were added.  
- **Reverse:** Reverse the order of tasks.  
- **Priority:** Sort tasks by priority.  
- **Alphanumeric:** Sort tasks in alphanumeric order.  
- **AlphanumericReverse:** Sort tasks in reverse alphanumeric order.  
</dd>
<br>

<dt><b>Flag:</b> <code>--done-sort</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_DONE_SORT</code></dt>
<dt><b>Conf:</b> <code>done_sort</code></dt>
<dd>
Specifies the sorting option for completed tasks.  

- **Possible Values (Flag):** `none`, `reverse`, `priority`, `alphanumeric`, `alphanumeric-reverse`  
- **Possible Values (Config):** `None`, `Reverse`, `Priority`, `Alphanumeric`, `AlphanumericReverse`  
- **Default:** `None`
</dd>
<br>

<dt><b>Flag:</b> <code>--delete-final-date</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_DELETE_FINAL_DATE</code></dt>
<dt><b>Conf:</b> <code>delete_final_date</code></dt>
<dd>
Specifies whether to delete the final date (if it exists) when a task is moved from completed back to pending.  

- **Default:** `true`
</dd>
<br>

<dt><b>Flag:</b> <code>--set-final-date</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_SET_FINAL_DATE</code></dt>
<dt><b>Conf:</b> <code>set_final_date</code></dt>
<dd>
Configures how the final date is handled when a task is marked as completed.  

- **Possible Values (Flag):** `override`, `only-missing`, `never`  
- **Possible Values (Config):** `Override`, `OnlyMissing`, `Never`  
- **Default:** `OnlyMissing`

**Final Date Options:**
- **Override:** Set the final date every time a task is marked as completed.  
- **OnlyMissing:** Set the final date only if it is not already set.  
- **Never:** Never set the final date. 
</dd>
<br>


<dt><b>Flag:</b> <code>--set-created-date</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_SET_CREATED_DATE</code></dt>
<dt><b>Conf:</b> <code>set_created_date</code></dt>
<dd>
Specifies whether to set the creation date when a new task is added. If the user provides their own creation date, it will still be added regardless of this setting.

- **Default:** `true`
</dd>
<br>
