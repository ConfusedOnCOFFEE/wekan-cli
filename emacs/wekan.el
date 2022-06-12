;; -*- lexical-binding: t -*-
;;
;; wekan.el is an emacs client to be used in combination with wekan-cli to
;; interact with a WekanBoard https://wekan.github.io/.
;; Copyright (C) 2022 Confused OnCoffee
;;
;; This program is free software: you can redistribute it and/or modify
;; it under the terms of the GNU Affero General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.
;;
;; This program is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU Affero General Public License for more details.
;;
;; You should have received a copy of the GNU Affero General Public License
;; along with this program.  If not, see <https://www.gnu.org/licenses/>.


(defcustom wekan-binary "wekan-cli"
"wekan binary name."
  :group 'wekan
  :type 'string)

(defcustom wekan-board "board"
"board command"
  :group 'wekan
    :type 'string)

(defun wekan-run-cmd (cmd args)
  "Run a wekan cmd."
  (let ((use-name (name cmd)))
    (start-process-shell-command
        use-name
        use-name
        (format "%s %s" wekan-binary use-name)
        )
      )
    )


;; (wekan-board-ls)
(defun wekan-board-ls ()
    (interactive)
    (wekan-run-cmd
        (wekan-binary)
        wekan-board
        "ls")
    )

(provide 'wekan)
;:: wekan.el ends here
